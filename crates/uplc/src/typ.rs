//! UPLC type system.
//!
//! [`Type`] represents the ground types of constants in a UPLC program.
//! It is used by built-in function dispatch to check and infer argument types at runtime.

use crate::arena::Arena;

/// A UPLC constant type.
///
/// These are the types that can appear as constant tags and in polymorphic built-in
/// function signatures. The machine uses them to type-check arguments at runtime.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    /// Boolean.
    Bool,
    /// Arbitrary-precision integer.
    Integer,
    /// UTF-8 string.
    String,
    /// Raw byte string.
    ByteString,
    /// Unit type `()`.
    Unit,
    /// Homogeneous list with the given element type.
    List(&'a Type<'a>),
    /// Homogeneous array with the given element type (Plutus V3).
    Array(&'a Type<'a>),
    /// Pair of two typed values.
    Pair(&'a Type<'a>, &'a Type<'a>),
    /// Plutus structured data.
    Data,
    /// BLS12-381 G1 curve point.
    Bls12_381G1Element,
    /// BLS12-381 G2 curve point.
    Bls12_381G2Element,
    /// BLS12-381 Miller-loop result.
    Bls12_381MlResult,
    Value,
}

impl<'a> Type<'a> {
    /// Allocates a [`Type::Integer`].
    pub fn integer(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Integer)
    }

    /// Allocates a [`Type::Bool`].
    pub fn bool(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bool)
    }

    /// Allocates a [`Type::String`].
    pub fn string(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::String)
    }

    /// Allocates a [`Type::ByteString`].
    pub fn byte_string(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::ByteString)
    }

    /// Allocates a [`Type::Unit`].
    pub fn unit(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Unit)
    }

    /// Allocates a [`Type::Data`].
    pub fn data(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Data)
    }

    /// Allocates a [`Type::List`] with the given element type.
    pub fn list(arena: &'a Arena, inner: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::List(inner))
    }

    /// Allocates a [`Type::Array`] with the given element type (Plutus V3).
    pub fn array(arena: &'a Arena, inner: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::Array(inner))
    }

    /// Allocates a [`Type::Pair`] with the given component types.
    pub fn pair(arena: &'a Arena, fst: &'a Type<'a>, snd: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::Pair(fst, snd))
    }

    /// Allocates a [`Type::Bls12_381G1Element`].
    pub fn g1(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381G1Element)
    }

    /// Allocates a [`Type::Bls12_381G2Element`].
    pub fn g2(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381G2Element)
    }

    /// Allocates a [`Type::Bls12_381MlResult`].
    pub fn ml_result(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381MlResult)
    }

    pub fn value(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Value)
    }
}
