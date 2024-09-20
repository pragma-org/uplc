use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::constant::{integer_from, Integer};

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

    pub fn byte_string(arena: &'a bumpalo::Bump, bytes: BumpVec<'a, u8>) -> &'a PlutusData<'a> {
        arena.alloc(PlutusData::ByteString(bytes))
    }
}
