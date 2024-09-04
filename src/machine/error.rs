use crate::{constant::Constant, term::Term, typ::Type};

use super::value::Value;

#[derive(Debug)]
pub enum MachineError<'a> {
    OpenTermEvaluated(&'a Term<'a>),
    ExplicitErrorTerm,
    NotAConstant(&'a Value<'a>),
    TypeMismatch(Type<'a>, &'a Constant<'a>),
}
