//! Evaluation result returned by the CEK machine.

use crate::{binder::Eval, term::Term};

use super::{info::MachineInfo, MachineError};

/// The result of evaluating a UPLC [`Program`](crate::program::Program).
#[must_use = "evaluation result must be inspected"]
#[derive(Debug)]
pub struct EvalResult<'a, V>
where
    V: Eval<'a>,
{
    /// The final reduced term, or the [`MachineError`] that halted evaluation.
    pub term: Result<&'a Term<'a, V>, MachineError<'a, V>>,
    /// Budget consumed and trace log lines produced during evaluation.
    pub info: MachineInfo,
}
