use bumpalo::Bump;

#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    Bool,
    Integer,
    String,
    ByteString,
    Unit,
    List(&'a Type<'a>),
    Pair(&'a Type<'a>, &'a Type<'a>),
    Data,
    Bls12_381G1Element,
    Bls12_381G2Element,
    Bls12_381MlResult,
}

impl<'a> Type<'a> {
    pub fn integer(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::Integer)
    }

    pub fn bool(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::Bool)
    }

    pub fn string(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::String)
    }

    pub fn byte_string(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::ByteString)
    }

    pub fn unit(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::Unit)
    }

    pub fn data(arena: &'a Bump) -> &'a Type<'a> {
        arena.alloc(Type::Data)
    }

    pub fn list(arena: &'a Bump, inner: &'a Type<'a>) -> &'a Type<'a> {
        arena.alloc(Type::List(inner))
    }
}
