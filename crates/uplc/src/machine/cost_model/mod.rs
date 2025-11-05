pub mod builtin_costs;
pub(crate) mod cost_map;
mod costing;
pub mod ex_budget;
mod machine_costs;
mod value;

pub use value::*;

use crate::machine::{
    cost_model::{builtin_costs::BuiltinCosts, machine_costs::MachineCosts},
    ExBudget, PlutusVersion,
};

#[derive(Default, Debug, PartialEq)]
pub struct CostModel {
    pub machine_startup: ExBudget,
    pub machine_costs: MachineCosts,
    pub builtin_costs: BuiltinCosts,
}

impl CostModel {
    pub fn initialize_cost_model(version: &PlutusVersion, cost_model: &[i64]) -> CostModel {
        let cost_map = cost_map::CostMap::new(version, cost_model);
        Self {
            machine_startup: ExBudget {
                mem: cost_map["cek_startup_cost-exBudgetmem"],
                cpu: cost_map["cek_startup_cost-exBudgetCPU"],
            },
            machine_costs: MachineCosts::initialize_machine_costs(&cost_map),
            builtin_costs: BuiltinCosts::initialize_builtin_costs(version, &cost_map),
        }
    }
}

impl From<&PlutusVersion> for CostModel {
    fn from(version: &PlutusVersion) -> Self {
        let builtin_costs = match version {
            crate::machine::PlutusVersion::V1 => BuiltinCosts::v1(),
            crate::machine::PlutusVersion::V2 => BuiltinCosts::v2(),
            crate::machine::PlutusVersion::V3 => BuiltinCosts::v3(),
        };
        Self {
            machine_startup: ExBudget::start_up(),
            machine_costs: Default::default(),
            builtin_costs,
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
