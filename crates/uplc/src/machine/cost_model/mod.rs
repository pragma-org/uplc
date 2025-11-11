pub mod builtin_costs;
pub(crate) mod cost_map;
mod costing;
pub mod ex_budget;
mod machine_costs;
mod value;

pub use value::*;

use crate::machine::{
    cost_model::{builtin_costs::BuiltinCostModel, machine_costs::MachineCosts},
    ExBudget, PlutusVersion,
};

#[derive(Debug, PartialEq)]
pub struct CostModel<B: BuiltinCostModel> {
    pub machine_startup: ExBudget,
    pub machine_costs: MachineCosts,
    pub builtin_costs: B,
}

impl<B: BuiltinCostModel> CostModel<B> {
    pub fn initialize_cost_model(version: &PlutusVersion, cost_model: &[i64]) -> CostModel<B> {
        let cost_map = cost_map::CostMap::new(version, cost_model);
        Self {
            machine_startup: ExBudget {
                mem: cost_map["cek_startup_cost-exBudgetmem"],
                cpu: cost_map["cek_startup_cost-exBudgetCPU"],
            },
            machine_costs: MachineCosts::initialize_machine_costs(&cost_map),
            builtin_costs: B::initialize(&cost_map),
        }
    }
}

impl<B: BuiltinCostModel + Default> Default for CostModel<B> {
    fn default() -> Self {
        Self {
            machine_startup: ExBudget::start_up(),
            machine_costs: Default::default(),
            builtin_costs: Default::default(),
        }
    }
}

#[repr(usize)]
pub enum StepKind {
    Constant = 0,
    Var = 1,
    Lambda = 2,
    Apply = 3,
    Delay = 4,
    Force = 5,
    Builtin = 6,
    Constr = 7,
    Case = 8,
}
