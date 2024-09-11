use crate::term::Term;

use super::MachineError;

pub struct EvalResult<'a> {
    pub result: Result<&'a Term<'a>, MachineError<'a>>,
}
