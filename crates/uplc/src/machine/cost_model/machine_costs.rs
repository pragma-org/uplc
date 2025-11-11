use crate::machine::{cost_model::cost_map::CostMap, ExBudget};

#[derive(Debug, PartialEq)]
pub struct MachineCosts([ExBudget; 9]);

impl Default for MachineCosts {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineCosts {
    pub fn new() -> Self {
        MachineCosts([
            ExBudget::constant(),
            ExBudget::var(),
            ExBudget::lambda(),
            ExBudget::apply(),
            ExBudget::delay(),
            ExBudget::force(),
            ExBudget::builtin(),
            ExBudget::constr(),
            ExBudget::case(),
        ])
    }

    pub fn get(&self, index: usize) -> ExBudget {
        self.0[index]
    }

    pub fn initialize_machine_costs(cost_map: &CostMap) -> Self {
        MachineCosts([
            ExBudget::new(
                cost_map["cek_const_cost-exBudgetmem"],
                cost_map["cek_const_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_var_cost-exBudgetmem"],
                cost_map["cek_var_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_lam_cost-exBudgetmem"],
                cost_map["cek_lam_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_apply_cost-exBudgetmem"],
                cost_map["cek_apply_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_delay_cost-exBudgetmem"],
                cost_map["cek_delay_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_force_cost-exBudgetmem"],
                cost_map["cek_force_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_builtin_cost-exBudgetmem"],
                cost_map["cek_builtin_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_constr_cost-exBudgetmem"],
                cost_map["cek_constr_cost-exBudgetCPU"],
            ),
            ExBudget::new(
                cost_map["cek_case_cost-exBudgetmem"],
                cost_map["cek_case_cost-exBudgetCPU"],
            ),
        ])
    }
}
