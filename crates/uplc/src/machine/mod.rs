//! CEK machine implementation and associated types.
//!
//! The main entry point is [`Program::eval`](crate::program::Program::eval). This module
//! re-exports the types needed to inspect evaluation results:
//!
//! - [`EvalResult`] — the final term (or error) plus [`MachineInfo`]
//! - [`MachineError`] — all error variants the machine can produce
//! - [`ExBudget`] — CPU/memory execution budget
//! - [`PlutusVersion`] — V1 / V2 / V3 semantics selector
//! - [`BuiltinSemantics`] — built-in behaviour variant (V1 or V2)
//! - [`CostModel`] — parameterised cost model

mod cek;
mod context;
pub(crate) mod cost_model;
mod discharge;
mod env;
mod error;
mod eval_result;
mod info;
mod runtime;
mod state;
mod value;

pub use cek::*;
pub use cost_model::ex_budget::*;
pub use cost_model::CostModel;
pub use error::*;
pub use eval_result::*;
pub use info::*;
pub use runtime::BuiltinSemantics;
pub use runtime::PlutusVersion;
