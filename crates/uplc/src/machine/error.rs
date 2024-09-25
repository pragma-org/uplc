use bumpalo::collections::Vec as BumpVec;

use crate::{
    constant::{Constant, Integer},
    term::Term,
    typ::Type,
};

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
    Runtime(RuntimeError<'a>),
}

#[derive(Debug)]
pub enum RuntimeError<'a> {
    ByteStringOutOfBounds(&'a BumpVec<'a, u8>, &'a Integer),
}

impl<'a> MachineError<'a> {
    pub fn runtime(runtime_error: RuntimeError<'a>) -> Self {
        MachineError::Runtime(runtime_error)
    }

    pub fn byte_string_out_of_bounds(byte_string: &'a BumpVec<'a, u8>, index: &'a Integer) -> Self {
        MachineError::runtime(RuntimeError::ByteStringOutOfBounds(byte_string, index))
    }
}
