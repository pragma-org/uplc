//! Execution budget tracking.
//!
//! [`ExBudget`] holds a CPU and memory allowance used by the CEK machine to bound evaluation.
//! The default budget mirrors the Cardano mainnet per-transaction limit.

/// Execution budget denominated in abstract memory and CPU units.
///
/// The default budget corresponds to the Cardano mainnet per-transaction limit
/// (`14_000_000` mem, `10_000_000_000` cpu). Use [`ExBudget::max`] for an effectively
/// unbounded budget in off-chain contexts.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExBudget {
    /// Memory units available or consumed.
    pub mem: i64,
    /// CPU step units available or consumed.
    pub cpu: i64,
}

impl Default for ExBudget {
    fn default() -> Self {
        Self::machine()
    }
}

impl ExBudget {
    /// Creates an [`ExBudget`] with explicit `mem` and `cpu` values.
    pub fn new(mem: i64, cpu: i64) -> Self {
        ExBudget { mem, cpu }
    }

    /// Returns an effectively unbounded budget for off-chain use.
    pub fn max() -> Self {
        Self::machine_max()
    }

    /// Scales both `mem` and `cpu` by `n` (used when a cost model has an occurrence factor).
    pub fn occurrences(&mut self, n: i64) {
        self.mem *= n;
        self.cpu *= n;
    }

    /// Cardano mainnet per-transaction budget (`14_000_000` mem, `10_000_000_000` cpu).
    pub fn machine() -> Self {
        ExBudget {
            mem: 14_000_000,
            cpu: 10_000_000_000,
        }
    }

    /// Effectively unbounded budget (`14_000_000_000_000` mem, `10_000_000_000_000_000` cpu).
    pub fn machine_max() -> Self {
        ExBudget {
            mem: 14_000_000_000_000,
            cpu: 10_000_000_000_000_000,
        }
    }

    /// Step cost charged once at program start-up.
    pub fn start_up() -> Self {
        ExBudget { mem: 100, cpu: 100 }
    }

    /// Step cost for a variable lookup (`Var`) step.
    pub fn var() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a constant (`Constant`) step.
    pub fn constant() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a lambda abstraction (`Lambda`) step.
    pub fn lambda() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a `Delay` step.
    pub fn delay() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a `Force` step.
    pub fn force() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a function application (`Apply`) step.
    pub fn apply() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for entering a built-in call (`Builtin`) step.
    pub fn builtin() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a constructor (`Constr`) step (Plutus V3).
    pub fn constr() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }

    /// Step cost for a `Case` step (Plutus V3).
    pub fn case() -> Self {
        ExBudget {
            mem: 100,
            cpu: 16000,
        }
    }
}

impl std::ops::Sub for ExBudget {
    type Output = ExBudget;

    fn sub(self, rhs: Self) -> Self::Output {
        ExBudget {
            mem: self.mem - rhs.mem,
            cpu: self.cpu - rhs.cpu,
        }
    }
}
