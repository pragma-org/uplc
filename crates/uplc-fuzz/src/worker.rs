use std::panic;

use crossbeam_channel::Sender;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use uplc_turbo::{
    arena::Arena,
    machine::{ExBudget, PlutusVersion},
};

use crate::{
    divergence::{compare_results, Divergence, DivergenceKind},
    eval::internal::{eval_bytecode, eval_cek, Budget, Outcome},
    gen::{builtin_aware::BuiltinAware, random::RandomStructural, Generator},
    seed::ProgramSeed,
};

/// Messages from worker to coordinator.
pub enum WorkerMessage {
    Divergence(Divergence),
    /// A seed that produced a non-error result (worth adding to corpus).
    CorpusEntry(ProgramSeed),
    /// Periodic stats update.
    Stats(WorkerStats),
}

#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub iterations: u64,
    pub successes: u64,
    pub errors: u64,
    pub divergences: u64,
    pub panics: u64,
}

pub struct WorkerConfig {
    pub id: usize,
    pub rng_seed: u64,
    pub plutus_version: PlutusVersion,
    pub budget: ExBudget,
    pub batch_size: usize,
    pub arena_reset_interval: usize,
    pub stats_interval: u64,
    pub version: (usize, usize, usize),
}

/// Run a worker loop. This is the hot path — designed for maximum throughput.
pub fn run_worker(config: WorkerConfig, tx: Sender<WorkerMessage>) {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(config.rng_seed);

    let random_gen = RandomStructural {
        max_depth: 12,
        version: config.version,
    };
    let builtin_gen = BuiltinAware {
        version: config.version,
    };

    let generators: Vec<(&dyn Generator, u32)> = vec![(&random_gen, 2), (&builtin_gen, 4)];
    let total_weight: u32 = generators.iter().map(|(_, w)| w).sum();

    let mut arena = Arena::new();
    let mut stats = WorkerStats::default();
    let mut iteration = 0u64;

    loop {
        // Select generator based on weights
        let roll = rand::Rng::gen_range(&mut rng, 0..total_weight);
        let mut cumulative = 0;
        let mut gen_idx = 0;
        for (i, (_, w)) in generators.iter().enumerate() {
            cumulative += w;
            if roll < cumulative {
                gen_idx = i;
                break;
            }
        }
        let (gen, _) = &generators[gen_idx];

        let batch = gen.generate_batch(&mut rng, config.batch_size);

        for seed in batch {
            let pv = config.plutus_version;
            let budget = config.budget;

            // Wrap both evaluations in catch_unwind to survive panics in either VM
            let eval_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                let program = seed.materialize(&arena);
                let cek_result = eval_cek(&arena, program, pv, budget);
                let bc_result = eval_bytecode(&arena, program, pv, budget);
                (cek_result, bc_result)
            }));

            match eval_result {
                Ok((cek_result, bc_result)) => {
                    // Compare results
                    if let Some(divergence) = compare_results(&cek_result, &bc_result, &seed) {
                        stats.divergences += 1;
                        let _ = tx.send(WorkerMessage::Divergence(divergence));
                    }

                    // Track whether the program was interesting (non-error)
                    match &cek_result.outcome {
                        Outcome::Success(_) => {
                            stats.successes += 1;
                            let _ = tx.send(WorkerMessage::CorpusEntry(seed));
                        }
                        _ => {
                            stats.errors += 1;
                        }
                    }
                }
                Err(e) => {
                    stats.panics += 1;
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    let _ = tx.send(WorkerMessage::Divergence(Divergence {
                        program: seed.clone(),
                        kind: DivergenceKind::Panic(msg),
                        cek_outcome: Outcome::EvaluationFailure("PANIC".to_string()),
                        bc_outcome: Outcome::EvaluationFailure("PANIC".to_string()),
                        cek_budget: Budget { cpu: 0, mem: 0 },
                        bc_budget: Budget { cpu: 0, mem: 0 },
                    }));
                    stats.divergences += 1;
                }
            }

            iteration += 1;
            stats.iterations += 1;

            // Reset arena periodically to avoid unbounded memory growth
            if iteration % config.arena_reset_interval as u64 == 0 {
                arena.reset();
            }
        }

        // Periodically send stats
        if iteration % config.stats_interval == 0 {
            let _ = tx.send(WorkerMessage::Stats(stats.clone()));
        }
    }
}
