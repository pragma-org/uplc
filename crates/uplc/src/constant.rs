use bumpalo::{collections::Vec as BumpVec, Bump};

#[derive(Debug, PartialEq)]
pub enum Constant<'a> {
    Integer(&'a Integer),
    ByteString(BumpVec<'a, u8>),
    Boolean(bool),
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

    pub fn integer_from(arena: &'a Bump, i: i128) -> &'a mut Constant {
        arena.alloc(Constant::Integer(integer_from(arena, i)))
    }

    pub fn unit(arena: &'a Bump) -> &'a mut Constant {
        arena.alloc(Constant::Unit)
    }
}