//! Constant values in UPLC programs.
//!
//! [`Constant`] covers all ground types: arbitrary-precision integers ([`Integer`]),
//! byte strings, UTF-8 strings, booleans, unit, homogeneous lists and arrays, pairs,
//! structured [`PlutusData`], and BLS12-381 curve elements.
//! 
use crate::{
    arena::Arena, binder::Eval, data::PlutusData, ledger_value::LedgerValue, machine::MachineError,
    typ::Type,
};

use crate::{arena::Arena, binder::Eval, data::PlutusData, machine::MachineError, typ::Type};

/// A UPLC ground-type constant.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Constant<'a> {
    /// Arbitrary-precision integer.
    Integer(&'a Integer),
    /// Raw byte string.
    ByteString(&'a [u8]),
    /// UTF-8 string.
    String(&'a str),
    /// Boolean.
    Boolean(bool),
    /// Plutus structured data.
    Data(&'a PlutusData<'a>),
    /// Homogeneous list.
    ProtoList(&'a Type<'a>, &'a [&'a Constant<'a>]),
    /// Homogeneous array (Plutus V3).
    ProtoArray(&'a Type<'a>, &'a [&'a Constant<'a>]),
    /// Pair of typed constants.
    ProtoPair(
        &'a Type<'a>,
        &'a Type<'a>,
        &'a Constant<'a>,
        &'a Constant<'a>,
    ),
    /// Unit value `()`.
    Unit,
    /// BLS12-381 G1 curve point.
    Bls12_381G1Element(&'a blst::blst_p1),
    /// BLS12-381 G2 curve point.
    Bls12_381G2Element(&'a blst::blst_p2),
    /// BLS12-381 Miller-loop result.
    Bls12_381MlResult(&'a blst::blst_fp12),
    Value(&'a LedgerValue<'a>),
}

/// Arbitrary-precision integer (alias for [`num::BigInt`]).
pub type Integer = num::BigInt;

/// Allocates a zero [`Integer`] in the arena.
pub fn integer(arena: &Arena) -> &Integer {
    arena.alloc_integer(Integer::default())
}

/// Allocates an [`Integer`] from an `i128`.
pub fn integer_from(arena: &Arena, i: i128) -> &Integer {
    arena.alloc_integer(Integer::from(i))
}

impl<'a> Constant<'a> {
    /// Allocates a [`Constant::Integer`].
    pub fn integer(arena: &'a Arena, i: &'a Integer) -> &'a Constant<'a> {
        arena.alloc(Constant::Integer(i))
    }

    /// Allocates a [`Constant::Integer`] from an `i128`.
    pub fn integer_from(arena: &'a Arena, i: i128) -> &'a Constant<'a> {
        arena.alloc(Constant::Integer(integer_from(arena, i)))
    }

    /// Allocates a [`Constant::ByteString`].
    pub fn byte_string(arena: &'a Arena, bytes: &'a [u8]) -> &'a Constant<'a> {
        arena.alloc(Constant::ByteString(bytes))
    }

    /// Allocates a [`Constant::String`].
    pub fn string(arena: &'a Arena, s: &'a str) -> &'a Constant<'a> {
        arena.alloc(Constant::String(s))
    }

    /// Allocates a [`Constant::Boolean`].
    pub fn bool(arena: &'a Arena, v: bool) -> &'a Constant<'a> {
        arena.alloc(Constant::Boolean(v))
    }

    /// Allocates a [`Constant::Data`].
    pub fn data(arena: &'a Arena, d: &'a PlutusData<'a>) -> &'a Constant<'a> {
        arena.alloc(Constant::Data(d))
    }

    /// Allocates a [`Constant::Unit`].
    pub fn unit(arena: &'a Arena) -> &'a Constant<'a> {
        arena.alloc(Constant::Unit)
    }

    /// Allocates a [`Constant::ProtoList`] with the given element type and values.
    pub fn proto_list(
        arena: &'a Arena,
        inner: &'a Type<'a>,
        values: &'a [&'a Constant<'a>],
    ) -> &'a Constant<'a> {
        arena.alloc(Constant::ProtoList(inner, values))
    }

    /// Allocates a [`Constant::ProtoArray`] with the given element type and values (Plutus V3).
    pub fn proto_array(
        arena: &'a Arena,
        inner: &'a Type<'a>,
        values: &'a [&'a Constant<'a>],
    ) -> &'a Constant<'a> {
        arena.alloc(Constant::ProtoArray(inner, values))
    }

    /// Allocates a [`Constant::ProtoPair`] with the given types and values.
    pub fn proto_pair(
        arena: &'a Arena,
        first_type: &'a Type<'a>,
        second_type: &'a Type<'a>,
        first_value: &'a Constant<'a>,
        second_value: &'a Constant<'a>,
    ) -> &'a Constant<'a> {
        arena.alloc(Constant::ProtoPair(
            first_type,
            second_type,
            first_value,
            second_value,
        ))
    }

    /// Allocates a [`Constant::Bls12_381G1Element`].
    pub fn g1(arena: &'a Arena, g1: &'a blst::blst_p1) -> &'a Constant<'a> {
        arena.alloc(Constant::Bls12_381G1Element(g1))
    }

    /// Allocates a [`Constant::Bls12_381G2Element`].
    pub fn g2(arena: &'a Arena, g2: &'a blst::blst_p2) -> &'a Constant<'a> {
        arena.alloc(Constant::Bls12_381G2Element(g2))
    }

    /// Allocates a [`Constant::Bls12_381MlResult`].
    pub fn ml_result(arena: &'a Arena, ml_res: &'a blst::blst_fp12) -> &'a Constant<'a> {
        arena.alloc(Constant::Bls12_381MlResult(ml_res))
    }

    pub fn ledger_value(arena: &'a Arena, v: &'a LedgerValue<'a>) -> &'a Constant<'a> {
        arena.alloc(Constant::Value(v))
    }

    pub fn unwrap_data<V>(&'a self) -> Result<&'a PlutusData<'a>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            Constant::Data(data) => Ok(data),
            _ => Err(MachineError::not_data(self)),
        }
    }

    /// Returns the runtime [`Type`] of this constant.
    pub fn type_of(&self, arena: &'a Arena) -> &'a Type<'a> {
        match self {
            Constant::Integer(_) => Type::integer(arena),
            Constant::ByteString(_) => Type::byte_string(arena),
            Constant::String(_) => Type::string(arena),
            Constant::Boolean(_) => Type::bool(arena),
            Constant::Data(_) => Type::data(arena),
            Constant::ProtoList(t, _) => Type::list(arena, t),
            Constant::ProtoArray(t, _) => Type::array(arena, t),
            Constant::ProtoPair(t1, t2, _, _) => Type::pair(arena, t1, t2),
            Constant::Unit => Type::unit(arena),
            Constant::Bls12_381G1Element(_) => Type::g1(arena),
            Constant::Bls12_381G2Element(_) => Type::g2(arena),
            Constant::Bls12_381MlResult(_) => Type::ml_result(arena),
            Constant::Value(_) => Type::value(arena),
        }
    }
}
