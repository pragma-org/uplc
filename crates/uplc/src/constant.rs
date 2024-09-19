use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};

use crate::data::PlutusData;

#[derive(Debug, PartialEq)]
pub enum Constant<'a> {
    Integer(&'a Integer),
    ByteString(BumpVec<'a, u8>),
    String(BumpString<'a>),
    Boolean(bool),
    Data(&'a PlutusData<'a>),
    Unit,
}

pub type Integer = rug::Integer;

pub fn integer(arena: &Bump) -> &mut Integer {
    arena.alloc(Integer::new())
}

pub fn integer_from(arena: &Bump, i: i128) -> &mut Integer {
    arena.alloc(Integer::from(i))
}

impl<'a> Constant<'a> {
    pub fn integer(arena: &'a Bump) -> &'a mut Constant {
        arena.alloc(Constant::Integer(integer(arena)))
    }

    pub fn integer_from(arena: &'a Bump, i: i128) -> &'a Constant {
        arena.alloc(Constant::Integer(integer_from(arena, i)))
    }

    pub fn byte_string(arena: &'a Bump, bytes: BumpVec<'a, u8>) -> &'a Constant<'a> {
        arena.alloc(Constant::ByteString(bytes))
    }

    pub fn string(arena: &'a Bump, s: BumpString<'a>) -> &'a Constant<'a> {
        arena.alloc(Constant::String(s))
    }

    pub fn bool(arena: &'a Bump, v: bool) -> &'a Constant<'a> {
        arena.alloc(Constant::Boolean(v))
    }

    pub fn data(arena: &'a Bump, d: &'a PlutusData<'a>) -> &'a Constant<'a> {
        arena.alloc(Constant::Data(d))
    }

    pub fn unit(arena: &'a Bump) -> &'a Constant {
        arena.alloc(Constant::Unit)
    }
}
