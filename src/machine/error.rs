use crate::term::Term;

pub enum MachineError<'a> {
    OpenTermEvaluated(&'a Term<'a>),
    ExplicitErrorTerm,
}
