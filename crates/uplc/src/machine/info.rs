//! Metadata captured during CEK machine evaluation.

use super::ExBudget;

/// Metadata captured during CEK machine evaluation.
#[derive(Debug)]
pub struct MachineInfo {
    /// Execution budget remaining after evaluation (initial budget minus consumed).
    pub consumed_budget: ExBudget,
    /// Lines emitted by `Trace` built-in calls, in order.
    pub logs: Vec<String>,
}
