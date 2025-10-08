pub mod builtin_costs;
mod costing;
pub mod ex_budget;
mod machine_costs;
mod value;

pub use value::*;

use crate::machine::PlutusVersion;

#[derive(Default, Debug, PartialEq)]
pub struct CostModel {
    pub machine_costs: machine_costs::MachineCosts,
    pub builtin_costs: builtin_costs::BuiltinCosts,
}

impl From<&PlutusVersion> for CostModel {
    fn from(version: &PlutusVersion) -> Self {
        let builtin_costs = match version {
            crate::machine::PlutusVersion::V1 => builtin_costs::BuiltinCosts::v1(),
            crate::machine::PlutusVersion::V2 => builtin_costs::BuiltinCosts::v2(),
            crate::machine::PlutusVersion::V3 => builtin_costs::BuiltinCosts::v3(),
        };
        Self {
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
