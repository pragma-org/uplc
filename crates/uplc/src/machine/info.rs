use super::ExBudget;

#[derive(Debug)]
pub struct MachineInfo {
    pub remaining_budget: ExBudget,
    pub consumed_budget: ExBudget,
    pub logs: Vec<String>,
}
