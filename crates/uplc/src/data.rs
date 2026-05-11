//! Plutus structured data.
//!
//! [`PlutusData`] is the serialisable data type passed across the Plutus script boundary.
//! It supports five constructors — `Constr`, `Map`, `List`, `Integer`, `ByteString` —
//! mirroring the Haskell `Data` type from `plutus-core`.

use crate::{
    arena::Arena,
    binder::Eval,
    constant::{integer_from, Constant, Integer},
    flat::Ctx,
    machine::MachineError,
};

/// Plutus structured data, serialisable across the script boundary.
///
/// This is the data type passed as datum, redeemer, and script context to on-chain
/// validators. It mirrors the Haskell `PlutusCore.Data` type.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum PlutusData<'a> {
    /// Tagged constructor with positional fields.
    Constr {
        /// Constructor tag (alternative index).
        tag: u64,
        /// Positional field values.
        fields: &'a [&'a PlutusData<'a>],
    },
    /// Association list (key-value map).
    Map(&'a [(&'a PlutusData<'a>, &'a PlutusData<'a>)]),
    /// Arbitrary-precision integer.
    Integer(&'a Integer),
    /// Raw byte string.
    ByteString(&'a [u8]),
    /// Homogeneous list.
    List(&'a [&'a PlutusData<'a>]),
}

impl<'a> PlutusData<'a> {
    /// Allocates a [`PlutusData::Constr`] with the given tag and fields.
    pub fn constr(
        arena: &'a Arena,
        tag: u64,
        fields: &'a [&'a PlutusData<'a>],
    ) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Constr { tag, fields })
    }

    /// Allocates a [`PlutusData::List`].
    pub fn list(arena: &'a Arena, items: &'a [&'a PlutusData<'a>]) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::List(items))
    }

    /// Allocates a [`PlutusData::Map`].
    pub fn map(
        arena: &'a Arena,
        items: &'a [(&'a PlutusData<'a>, &'a PlutusData<'a>)],
    ) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Map(items))
    }

    /// Allocates a [`PlutusData::Integer`].
    pub fn integer(arena: &'a Arena, i: &'a Integer) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Integer(i))
    }

    /// Allocates a [`PlutusData::Integer`] from an `i128`.
    pub fn integer_from(arena: &'a Arena, i: i128) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Integer(integer_from(arena, i)))
    }

    /// Allocates a [`PlutusData::ByteString`].
    pub fn byte_string(arena: &'a Arena, bytes: &'a [u8]) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::ByteString(bytes))
    }

    /// Decodes a CBOR-encoded `PlutusData` value.
    pub fn from_cbor(
        arena: &'a Arena,
        cbor: &'_ [u8],
    ) -> Result<&'a PlutusData<'a>, minicbor::decode::Error> {
        minicbor::decode_with(cbor, &mut Ctx { arena })
    }

    /// Unwraps a [`PlutusData::Constr`], returning `(tag, fields)`.
    pub fn unwrap_constr<V>(
        &'a self,
    ) -> Result<(&'a u64, &'a [&'a PlutusData<'a>]), MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            PlutusData::Constr { tag, fields } => Ok((tag, fields)),
            _ => Err(MachineError::malformed_data(self)),
        }
    }

    /// Unwraps a [`PlutusData::Map`].
    pub fn unwrap_map<V>(
        &'a self,
    ) -> Result<&'a [(&'a PlutusData<'a>, &'a PlutusData<'a>)], MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            PlutusData::Map(fields) => Ok(fields),
            _ => Err(MachineError::malformed_data(self)),
        }
    }

    /// Unwraps a [`PlutusData::Integer`].
    pub fn unwrap_integer<V>(&'a self) -> Result<&'a Integer, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            PlutusData::Integer(i) => Ok(i),
            _ => Err(MachineError::malformed_data(self)),
        }
    }

    /// Unwraps a [`PlutusData::ByteString`].
    pub fn unwrap_byte_string<V>(&'a self) -> Result<&'a [u8], MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            PlutusData::ByteString(bytes) => Ok(bytes),
            _ => Err(MachineError::malformed_data(self)),
        }
    }

    /// Unwraps a [`PlutusData::List`].
    pub fn unwrap_list<V>(&'a self) -> Result<&'a [&'a PlutusData<'a>], MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match self {
            PlutusData::List(items) => Ok(items),
            _ => Err(MachineError::malformed_data(self)),
        }
    }

    /// Wraps this value in a [`Constant::Data`].
    pub fn constant(&'a self, arena: &'a Arena) -> &'a Constant<'a> {
        Constant::data(arena, self)
    }

    /// CBOR-serialises this value into a byte slice allocated in the arena.
    pub fn to_bytes<V>(&'a self, arena: &'a Arena) -> Result<&'a [u8], MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        minicbor::to_vec(self)
            .map(|vec| arena.alloc(vec) as &'a [u8])
            .map_err(|_| MachineError::serialization_error(self))
    }
}
