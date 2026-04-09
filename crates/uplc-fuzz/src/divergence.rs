use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

use crate::{
    eval::internal::{Budget, EngineResult, Outcome},
    seed::ProgramSeed,
};

/// A divergence between two evaluation engines.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Divergence {
    pub program: ProgramSeed,
    pub kind: DivergenceKind,
    pub cek_outcome: Outcome,
    pub bc_outcome: Outcome,
    pub cek_budget: Budget,
    pub bc_budget: Budget,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DivergenceKind {
    /// The two engines produced different results (one succeeded, one failed, or different values).
    ResultMismatch,
    /// Both engines produced the same result but consumed different budgets.
    BudgetMismatch,
    /// Both engines produced different results AND different budgets.
    ResultAndBudgetMismatch,
    /// The bytecode compiler or VM panicked.
    Panic(String),
    /// An external engine diverged from the internal engines.
    ExternalMismatch {
        external_output: String,
    },
}

/// Compare CEK and bytecode results, returning a divergence if they differ.
pub fn compare_results(
    cek: &EngineResult,
    bc: &EngineResult,
    seed: &ProgramSeed,
) -> Option<Divergence> {
    let result_matches = match (&cek.outcome, &bc.outcome) {
        (Outcome::Success(a), Outcome::Success(b)) => a == b,
        (Outcome::EvaluationFailure(_), Outcome::EvaluationFailure(_)) => true,
        (Outcome::BudgetExceeded, Outcome::BudgetExceeded) => true,
        _ => false,
    };

    let budget_matches = cek.budget == bc.budget;

    if result_matches && budget_matches {
        return None;
    }

    let kind = match (result_matches, budget_matches) {
        (false, false) => DivergenceKind::ResultAndBudgetMismatch,
        (false, true) => DivergenceKind::ResultMismatch,
        (true, false) => DivergenceKind::BudgetMismatch,
        (true, true) => unreachable!(),
    };

    Some(Divergence {
        program: seed.clone(),
        kind,
        cek_outcome: cek.outcome.clone(),
        bc_outcome: bc.outcome.clone(),
        cek_budget: cek.budget,
        bc_budget: bc.budget,
    })
}

/// Append-only on-disk catalog of divergences with in-memory dedup.
pub struct DivergenceCatalog {
    path: PathBuf,
    seen: Mutex<HashSet<u64>>,
    count: std::sync::atomic::AtomicUsize,
}

impl DivergenceCatalog {
    pub fn new(path: PathBuf) -> Self {
        fs::create_dir_all(&path).ok();
        Self {
            path,
            seen: Mutex::new(HashSet::new()),
            count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Record a divergence. Returns true if it was new (not a duplicate).
    pub fn record(&self, divergence: &Divergence) -> bool {
        let hash = hash_seed(&divergence.program);

        {
            let mut seen = self.seen.lock().unwrap();
            if seen.contains(&hash) {
                return false;
            }
            seen.insert(hash);
        }

        let idx = self
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Write JSON-lines entry
        let jsonl_path = self.path.join("catalog.jsonl");
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&jsonl_path)
        {
            if let Ok(json) = serde_json::to_string(divergence) {
                let _ = writeln!(f, "{json}");
            }
        }

        // Write standalone .uplc file
        let uplc_path = self.path.join(format!("divergence_{idx:04}.uplc"));
        if let Ok(mut f) = fs::File::create(&uplc_path) {
            let _ = writeln!(f, "-- Divergence: {:?}", divergence.kind);
            let _ = writeln!(f, "-- CEK outcome: {:?}", divergence.cek_outcome);
            let _ = writeln!(f, "-- CEK budget: cpu={} mem={}", divergence.cek_budget.cpu, divergence.cek_budget.mem);
            let _ = writeln!(f, "-- BC outcome: {:?}", divergence.bc_outcome);
            let _ = writeln!(f, "-- BC budget: cpu={} mem={}", divergence.bc_budget.cpu, divergence.bc_budget.mem);
            let _ = writeln!(f, "{}", divergence.program);
        }

        true
    }

    pub fn count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn hash_seed(seed: &ProgramSeed) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = xxhash_rust::xxh3::Xxh3Default::default();
    seed.hash(&mut hasher);
    hasher.finish()
}
