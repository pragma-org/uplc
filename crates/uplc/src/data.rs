use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::{
    constant::{integer_from, Integer},
    flat::decode::Ctx,
    machine::MachineError,
};

#[derive(Debug, PartialEq)]
pub enum PlutusData<'a> {
    Constr {
        tag: u64,
        fields: BumpVec<'a, &'a PlutusData<'a>>,
    },
    Map(BumpVec<'a, (&'a PlutusData<'a>, &'a PlutusData<'a>)>),
    Integer(&'a Integer),
    ByteString(BumpVec<'a, u8>),
    List(BumpVec<'a, &'a PlutusData<'a>>),
}

impl<'a> PlutusData<'a> {
    pub fn constr(
        arena: &'a Bump,
        tag: u64,
        fields: BumpVec<'a, &'a PlutusData<'a>>,
    ) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Constr { tag, fields })
    }

    pub fn list(arena: &'a Bump, items: BumpVec<'a, &'a PlutusData<'a>>) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::List(items))
    }

    pub fn map(
        arena: &'a Bump,
        items: BumpVec<'a, (&'a PlutusData<'a>, &'a PlutusData<'a>)>,
    ) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Map(items))
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Integer(i))
    }

    pub fn integer_from(arena: &'a Bump, i: i128) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::Integer(integer_from(arena, i)))
    }

    pub fn byte_string(arena: &'a Bump, bytes: BumpVec<'a, u8>) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::ByteString(bytes))
    }

    pub fn from_cbor(
        arena: &'a Bump,
        cbor: &'_ [u8],
    ) -> Result<&'a PlutusData<'a>, minicbor::decode::Error> {
        minicbor::decode_with(cbor, &mut Ctx { arena })
    }

    pub fn unwrap_constr(
        &'a self,
    ) -> Result<(&'a u64, &'a BumpVec<&'a PlutusData<'a>>), MachineError<'a>> {
        match self {
            PlutusData::Constr { tag, fields } => Ok((tag, fields)),
            _ => Err(MachineError::malformed_data(self)),
        }
    }
}
