use std::array::TryFromSliceError;

use bumpalo::collections::Vec as BumpVec;

use crate::{
    constant::{Constant, Integer},
    data::PlutusData,
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
    TypeMismatch(Type<'a>, &'a Constant<'a>),
    ExpectedPair(&'a Constant<'a>),
    ExpectedList(&'a Constant<'a>),
    NotData(&'a Constant<'a>),
    MalFormedData(&'a PlutusData<'a>),
    EmptyList(&'a BumpVec<'a, &'a Constant<'a>>),
    UnexpectedEd25519PublicKeyLength(TryFromSliceError),
    UnexpectedEd25519SignatureLength(TryFromSliceError),
}

impl<'a> MachineError<'a> {
    pub fn runtime(runtime_error: RuntimeError<'a>) -> Self {
        MachineError::Runtime(runtime_error)
    }

    pub fn type_mismatch(expected: Type<'a>, constant: &'a Constant<'a>) -> Self {
        MachineError::runtime(RuntimeError::TypeMismatch(expected, constant))
    }

    pub fn expected_pair(constant: &'a Constant<'a>) -> Self {
        MachineError::runtime(RuntimeError::ExpectedPair(constant))
    }

    pub fn expected_list(constant: &'a Constant<'a>) -> Self {
        MachineError::runtime(RuntimeError::ExpectedList(constant))
    }

    pub fn empty_list(constant: &'a BumpVec<'a, &'a Constant<'a>>) -> Self {
        MachineError::runtime(RuntimeError::EmptyList(constant))
    }

    pub fn byte_string_out_of_bounds(byte_string: &'a BumpVec<'a, u8>, index: &'a Integer) -> Self {
        MachineError::runtime(RuntimeError::ByteStringOutOfBounds(byte_string, index))
    }

    pub fn not_data(constant: &'a Constant<'a>) -> Self {
        MachineError::runtime(RuntimeError::NotData(constant))
    }

    pub fn malformed_data(plutus_data: &'a PlutusData<'a>) -> Self {
        MachineError::runtime(RuntimeError::MalFormedData(plutus_data))
    }

    pub fn unexpected_ed25519_public_key_length(length: TryFromSliceError) -> Self {
        MachineError::runtime(RuntimeError::UnexpectedEd25519PublicKeyLength(length))
    }

    pub fn unexpected_ed25519_signature_length(length: TryFromSliceError) -> Self {
        MachineError::runtime(RuntimeError::UnexpectedEd25519SignatureLength(length))
    }
}
