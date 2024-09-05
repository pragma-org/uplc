use crate::{constant::Constant, term::Term, typ::Type};

use super::value::Value;

#[derive(Debug)]
pub enum MachineError<'a> {
    ExplicitErrorTerm,
    NonFunctionApplication(&'a Value<'a>, &'a Value<'a>),
    NotAConstant(&'a Value<'a>),
    OpenTermEvaluated(&'a Term<'a>),
    TypeMismatch(Type<'a>, &'a Constant<'a>),
    UnexpectedBuiltinTermArgument(&'a Term<'a>),
}
