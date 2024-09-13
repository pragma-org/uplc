use crate::term::Term;

use super::{info::MachineInfo, MachineError};

pub struct EvalResult<'a> {
    pub term: Result<&'a Term<'a>, MachineError<'a>>,
    pub info: MachineInfo,
}
