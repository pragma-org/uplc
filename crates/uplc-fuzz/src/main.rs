use std::{
    path::{Path, PathBuf},
    sync::{atomic::Ordering, Arc},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use crossbeam_channel::bounded;
use uplc_turbo::machine::{ExBudget, PlutusVersion};

use uplc_fuzz::{
    divergence::DivergenceCatalog,
    eval::internal::Outcome,
    seed::ProgramSeed,
    stats::Stats,
    tui::{Tui, TuiState},
    worker::{WorkerConfig, WorkerMessage},
};

#[derive(Parser)]
#[command(
    name = "uplc-fuzz",
    about = "Differential fuzzer for UPLC virtual machines"
)]
struct Cli {
    /// Number of worker threads [default: number of CPUs]
    #[arg(short = 'j', long)]
    jobs: Option<usize>,

    /// Run for this many seconds (0 = unlimited)
    #[arg(short = 'd', long, default_value = "0")]
    duration: u64,

    /// Output directory for divergence catalog
    #[arg(short = 'o', long, default_value = "fuzz-output")]
    output: PathBuf,

    /// RNG seed for reproducibility (0 = random)
    #[arg(long, default_value = "0")]
    seed: u64,

    /// Plutus version: v1, v2, v3
    #[arg(long, default_value = "v3")]
    version: String,

    /// CPU budget limit
    #[arg(long, default_value = "10000000000")]
    budget_cpu: i64,

    /// Memory budget limit
    #[arg(long, default_value = "14000000")]
    budget_mem: i64,

    /// Batch size per generator invocation
    #[arg(long, default_value = "64")]
    batch_size: usize,

    /// External harness command (e.g., "uplc evaluate")
    #[arg(long)]
    external: Option<String>,

    /// Replay a specific divergence file (no TUI)
    #[arg(long)]
    replay: Option<PathBuf>,

    /// Disable TUI, use plain log output
    #[arg(long)]
    no_tui: bool,
}

fn parse_plutus_version(s: &str) -> PlutusVersion {
    match s {
        "v1" => PlutusVersion::V1,
        "v2" => PlutusVersion::V2,
        _ => PlutusVersion::V3,
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(replay_path) = &cli.replay {
        replay(replay_path, &cli);
        return;
    }

    let num_workers = cli.jobs.unwrap_or_else(num_cpus::get);
    let plutus_version = parse_plutus_version(&cli.version);
    let program_version = match plutus_version {
        PlutusVersion::V1 | PlutusVersion::V2 => (1, 0, 0),
        PlutusVersion::V3 => (1, 1, 0),
    };
    let budget = ExBudget::new(cli.budget_mem, cli.budget_cpu);

    let base_seed = if cli.seed == 0 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    } else {
        cli.seed
    };

    let catalog = Arc::new(DivergenceCatalog::new(cli.output.clone()));
    let stats = Arc::new(Stats::new());
    let (tx, rx) = bounded::<WorkerMessage>(num_workers * 256);

    // Spawn worker threads
    for i in 0..num_workers {
        let tx = tx.clone();
        let config = WorkerConfig {
            id: i,
            rng_seed: base_seed.wrapping_add((i as u64).wrapping_mul(0x9E3779B97F4A7C15)),
            plutus_version,
            budget,
            batch_size: cli.batch_size,
            arena_reset_interval: 256,
            stats_interval: 1000,
            version: program_version,
        };
        thread::Builder::new()
            .name(format!("worker-{i}"))
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                uplc_fuzz::worker::run_worker(config, tx);
            })
            .expect("failed to spawn worker thread");
    }
    drop(tx);

    let duration = if cli.duration > 0 {
        Duration::from_secs(cli.duration)
    } else {
        Duration::from_secs(u64::MAX)
    };

    if cli.no_tui {
        run_plain(&rx, &stats, &catalog, duration);
    } else {
        run_tui(
            &rx,
            &stats,
            &catalog,
            duration,
            num_workers,
            &cli.version,
            &cli.output,
            base_seed,
        );
    }
}

fn run_tui(
    rx: &crossbeam_channel::Receiver<WorkerMessage>,
    stats: &Arc<Stats>,
    catalog: &Arc<DivergenceCatalog>,
    duration: Duration,
    num_workers: usize,
    plutus_version: &str,
    output_dir: &Path,
    base_seed: u64,
) {
    let mut tui = match Tui::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize TUI: {e}. Falling back to plain output.");
            run_plain(rx, stats, catalog, duration);
            return;
        }
    };

    let mut state = TuiState::new(
        num_workers,
        plutus_version,
        &output_dir.display().to_string(),
        base_seed,
    );

    let start = Instant::now();
    let mut last_draw = Instant::now();
    let mut corpus: Vec<ProgramSeed> = Vec::new();
    let corpus_max = 10_000;

    loop {
        if start.elapsed() >= duration {
            break;
        }

        if tui.poll_quit() {
            break;
        }

        // Drain channel
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMessage::Divergence(d) => {
                    let is_new = catalog.record(&d);
                    stats.divergences.fetch_add(1, Ordering::Relaxed);
                    state.record_divergence_kind(&d.kind);
                    if is_new {
                        state.push_divergence_message(format!(
                            "#{} {:?} (nodes={})",
                            catalog.count(),
                            d.kind,
                            d.program.node_count()
                        ));
                    }
                }
                WorkerMessage::CorpusEntry(seed) => {
                    if corpus.len() < corpus_max {
                        corpus.push(seed);
                    }
                    stats
                        .corpus_size
                        .store(corpus.len() as u64, Ordering::Relaxed);
                }
                WorkerMessage::Stats(ws) => {
                    stats.iterations.fetch_add(ws.iterations, Ordering::Relaxed);
                    stats.successes.fetch_add(ws.successes, Ordering::Relaxed);
                    stats.errors.fetch_add(ws.errors, Ordering::Relaxed);
                    stats.panics.fetch_add(ws.panics, Ordering::Relaxed);
                }
            }
        }

        // Redraw at ~15 fps
        if last_draw.elapsed() >= Duration::from_millis(66) {
            state.sample_throughput(stats);
            if tui.draw(stats, &state, catalog.count()).is_err() {
                break;
            }
            last_draw = Instant::now();
        }

        // Small sleep to not spin-loop
        thread::sleep(Duration::from_millis(10));
    }

    tui.restore().ok();

    // Print final summary to stderr
    eprintln!();
    stats.print_summary();
    eprintln!("Total unique divergences: {}", catalog.count());
    eprintln!("  Result mismatches:  {}", state.result_mismatches);
    eprintln!("  Budget mismatches:  {}", state.budget_mismatches);
    eprintln!("  Result+Budget:      {}", state.result_and_budget);
    eprintln!("  Panics:             {}", state.panics_count);
    eprintln!("Catalog: {}", output_dir.display());
}

fn run_plain(
    rx: &crossbeam_channel::Receiver<WorkerMessage>,
    stats: &Arc<Stats>,
    catalog: &Arc<DivergenceCatalog>,
    duration: Duration,
) {
    let start = Instant::now();
    let mut last_stats = Instant::now();
    let mut corpus: Vec<ProgramSeed> = Vec::new();
    let corpus_max = 10_000;

    loop {
        if start.elapsed() >= duration {
            eprintln!("\nDuration limit reached.");
            break;
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(WorkerMessage::Divergence(d)) => {
                let is_new = catalog.record(&d);
                stats.divergences.fetch_add(1, Ordering::Relaxed);
                if is_new {
                    eprintln!(
                        "  DIVERGENCE #{}: {:?} (nodes={})",
                        catalog.count(),
                        d.kind,
                        d.program.node_count()
                    );
                }
            }
            Ok(WorkerMessage::CorpusEntry(seed)) => {
                if corpus.len() < corpus_max {
                    corpus.push(seed);
                }
                stats
                    .corpus_size
                    .store(corpus.len() as u64, Ordering::Relaxed);
            }
            Ok(WorkerMessage::Stats(ws)) => {
                stats.iterations.fetch_add(ws.iterations, Ordering::Relaxed);
                stats.successes.fetch_add(ws.successes, Ordering::Relaxed);
                stats.errors.fetch_add(ws.errors, Ordering::Relaxed);
                stats.panics.fetch_add(ws.panics, Ordering::Relaxed);
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                eprintln!("\nAll workers finished.");
                break;
            }
        }

        if last_stats.elapsed() >= Duration::from_secs(5) {
            stats.print_summary();
            last_stats = Instant::now();
        }
    }

    stats.print_summary();
    eprintln!("\nTotal divergences found: {}", catalog.count());
}

fn replay(path: &Path, cli: &Cli) {
    use uplc_fuzz::eval::internal::{eval_bytecode, eval_cek};
    use uplc_turbo::arena::Arena;

    let plutus_version = parse_plutus_version(&cli.version);
    let budget = ExBudget::new(cli.budget_mem, cli.budget_cpu);

    let contents = std::fs::read_to_string(path).expect("failed to read file");
    let uplc_text: String = contents
        .lines()
        .filter(|l| !l.starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");

    let debug_replay = std::env::var("UPLC_FUZZ_DEBUG_REPLAY").is_ok();

    let arena = Arena::new();
    let program = uplc_turbo::syn::parse_program(&arena, &uplc_text)
        .into_result()
        .expect("failed to parse UPLC program");

    eprintln!("Replaying: {}", path.display());
    if debug_replay {
        eprintln!("CEK top-level: {}", term_variant_name(program.term));
    }

    let cek_result = eval_cek(&arena, program, plutus_version, budget);
    eprintln!("CEK outcome:  {:?}", cek_result.outcome);
    eprintln!(
        "CEK budget:   cpu={} mem={}",
        cek_result.budget.cpu, cek_result.budget.mem
    );

    let arena2 = Arena::new();
    let program2 = uplc_turbo::syn::parse_program(&arena2, &uplc_text)
        .into_result()
        .unwrap();
    if debug_replay {
        eprintln!("BC top-level:  {}", term_variant_name(program2.term));
    }

    let bc_result = eval_bytecode(&arena2, program2, plutus_version, budget);
    eprintln!("BC outcome:   {:?}", bc_result.outcome);
    eprintln!(
        "BC budget:    cpu={} mem={}",
        bc_result.budget.cpu, bc_result.budget.mem
    );

    let result_matches = match (&cek_result.outcome, &bc_result.outcome) {
        (Outcome::Success(a), Outcome::Success(b)) => a == b,
        (Outcome::EvaluationFailure(_), Outcome::EvaluationFailure(_)) => true,
        (Outcome::BudgetExceeded, Outcome::BudgetExceeded) => true,
        _ => false,
    };

    let budget_matches = cek_result.budget == bc_result.budget;

    if result_matches && budget_matches {
        eprintln!("\nResults MATCH.");
    } else {
        eprintln!("\nResults DIVERGE!");
        if !result_matches {
            eprintln!("  Outcome differs.");
        } else if let (Outcome::EvaluationFailure(a), Outcome::EvaluationFailure(b)) =
            (&cek_result.outcome, &bc_result.outcome)
        {
            if a != b {
                eprintln!("  Both failed (different errors):");
                eprintln!("    CEK: {a}");
                eprintln!("    BC:  {b}");
            }
        }

        if !budget_matches {
            eprintln!(
                "  Budget differs: CEK cpu={} mem={}, BC cpu={} mem={}",
                cek_result.budget.cpu,
                cek_result.budget.mem,
                bc_result.budget.cpu,
                bc_result.budget.mem
            );
        }

        std::process::exit(1);
    }
}

fn term_variant_name(
    term: &uplc_turbo::term::Term<'_, uplc_turbo::binder::DeBruijn>,
) -> &'static str {
    use uplc_turbo::term::Term;
    match term {
        Term::Var(_) => "Var",
        Term::Lambda { .. } => "Lambda",
        Term::Apply { .. } => "Apply",
        Term::Constant(_) => "Constant",
        Term::Force(_) => "Force",
        Term::Delay(_) => "Delay",
        Term::Error => "Error",
        Term::Builtin(_) => "Builtin",
        Term::Constr { .. } => "Constr",
        Term::Case { .. } => "Case",
    }
}
