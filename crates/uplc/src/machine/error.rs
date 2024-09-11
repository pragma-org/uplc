use crate::{constant::Constant, term::Term, typ::Type};

use super::{value::Value, ExBudget};

#[derive(Debug)]
pub enum MachineError<'a> {
    ExplicitErrorTerm,
    NonFunctionApplication(&'a Value<'a>, &'a Value<'a>),
    NotAConstant(&'a Value<'a>),
    OpenTermEvaluated(&'a Term<'a>),
    OutOfExError(ExBudget),
    TypeMismatch(Type<'a>, &'a Constant<'a>),
    UnexpectedBuiltinTermArgument(&'a Term<'a>),
    NonPolymorphicInstantiation(&'a Value<'a>),
    BuiltinTermArgumentExpected(&'a Term<'a>),
    NonConstrScrutinized(&'a Value<'a>),
    MissingCaseBranch(&'a [&'a Term<'a>], &'a Value<'a>),
}
