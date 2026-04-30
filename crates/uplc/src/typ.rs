use crate::arena::Arena;

#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    Bool,
    Integer,
    String,
    ByteString,
    Unit,
    List(&'a Type<'a>),
    Array(&'a Type<'a>),
    Pair(&'a Type<'a>, &'a Type<'a>),
    Data,
    Bls12_381G1Element,
    Bls12_381G2Element,
    Bls12_381MlResult,
    Value,
}

impl<'a> Type<'a> {
    pub fn integer(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Integer)
    }

    pub fn bool(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bool)
    }

    pub fn string(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::String)
    }

    pub fn byte_string(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::ByteString)
    }

    pub fn unit(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Unit)
    }

    pub fn data(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Data)
    }

    pub fn list(arena: &'a Arena, inner: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::List(inner))
    }

    pub fn array(arena: &'a Arena, inner: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::Array(inner))
    }

    pub fn pair(arena: &'a Arena, fst: &'a Type<'a>, snd: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::Pair(fst, snd))
    }

    pub fn g1(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381G1Element)
    }

    pub fn g2(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381G2Element)
    }

    pub fn ml_result(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Bls12_381MlResult)
    }

    pub fn value(arena: &'a Arena) -> &'a Type<'a> {
        arena.alloc(Type::Value)
    }
}
