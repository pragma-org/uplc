use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};

use crate::{
    builtin::DefaultFunction,
    constant::{integer_from, Constant, Integer},
    data::PlutusData,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Term<'a> {
    Var(usize),

    Lambda {
        parameter: usize,
        body: &'a Term<'a>,
    },

    Apply {
        function: &'a Term<'a>,
        argument: &'a Term<'a>,
    },

    Delay(&'a Term<'a>),

    Force(&'a Term<'a>),

    Case {
        constr: &'a Term<'a>,
        branches: BumpVec<'a, &'a Term<'a>>,
    },

    Constr {
        // TODO: revisit what the best type is for this
        tag: usize,
        fields: BumpVec<'a, &'a Term<'a>>,
    },

    Constant(&'a Constant<'a>),

    Builtin(&'a DefaultFunction),

    Error,
}

impl<'a> Term<'a> {
    pub fn var(arena: &'a Bump, i: usize) -> &'a Term<'a> {
        arena.alloc(Term::Var(i))
    }

    pub fn apply(&'a self, arena: &'a Bump, argument: &'a Term<'a>) -> &'a Term<'a> {
        arena.alloc(Term::Apply {
            function: self,
            argument,
        })
    }

    pub fn lambda(&'a self, arena: &'a Bump, parameter: usize) -> &'a Term<'a> {
        arena.alloc(Term::Lambda {
            parameter,
            body: self,
        })
    }

    pub fn force(&'a self, arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Force(self))
    }

    pub fn delay(&'a self, arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Delay(self))
    }

    pub fn constant(arena: &'a Bump, constant: &'a Constant<'a>) -> &'a Term<'a> {
        arena.alloc(Term::Constant(constant))
    }

    pub fn constr(arena: &'a Bump, tag: usize, fields: BumpVec<'a, &'a Term<'a>>) -> &'a Term<'a> {
        arena.alloc(Term::Constr { tag, fields })
    }

    pub fn case(
        arena: &'a Bump,
        constr: &'a Term<'a>,
        branches: BumpVec<'a, &'a Term<'a>>,
    ) -> &'a Term<'a> {
        arena.alloc(Term::Case { constr, branches })
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a Term<'a> {
        let constant = arena.alloc(Constant::Integer(i));

        Term::constant(arena, constant)
    }

    pub fn integer_from(arena: &'a Bump, i: i128) -> &'a Term<'a> {
        Self::integer(arena, integer_from(arena, i))
    }

    pub fn byte_string(arena: &'a Bump, bytes: BumpVec<'a, u8>) -> &'a Term<'a> {
        let constant = Constant::byte_string(arena, bytes);

        Term::constant(arena, constant)
    }

    pub fn string(arena: &'a Bump, s: BumpString<'a>) -> &'a Term<'a> {
        let constant = Constant::string(arena, s);

        Term::constant(arena, constant)
    }

    pub fn bool(arena: &'a Bump, v: bool) -> &'a Term<'a> {
        let constant = Constant::bool(arena, v);

        Term::constant(arena, constant)
    }

    pub fn data(arena: &'a Bump, d: &'a PlutusData<'a>) -> &'a Term<'a> {
        let constant = Constant::data(arena, d);

        Term::constant(arena, constant)
    }

    pub fn data_byte_string(arena: &'a Bump, bytes: BumpVec<'a, u8>) -> &'a Term<'a> {
        let data = PlutusData::byte_string(arena, bytes);

        Term::data(arena, data)
    }

    pub fn data_integer(arena: &'a Bump, i: &'a Integer) -> &'a Term<'a> {
        let data = PlutusData::integer(arena, i);

        Term::data(arena, data)
    }

    pub fn data_integer_from(arena: &'a Bump, i: i128) -> &'a Term<'a> {
        let data = PlutusData::integer_from(arena, i);

        Term::data(arena, data)
    }

    pub fn unit(arena: &'a Bump) -> &'a Term<'a> {
        let constant = Constant::unit(arena);

        Term::constant(arena, constant)
    }

    pub fn builtin(arena: &'a Bump, fun: &'a DefaultFunction) -> &'a Term<'a> {
        arena.alloc(Term::Builtin(fun))
    }

    pub fn error(arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Error)
    }

    pub fn add_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::AddInteger);

        Term::builtin(arena, fun)
    }

    pub fn multiply_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::MultiplyInteger);

        Term::builtin(arena, fun)
    }

    pub fn divide_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::DivideInteger);

        Term::builtin(arena, fun)
    }

    pub fn quotient_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::QuotientInteger);

        Term::builtin(arena, fun)
    }

    pub fn remainder_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::RemainderInteger);

        Term::builtin(arena, fun)
    }

    pub fn mod_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::ModInteger);

        Term::builtin(arena, fun)
    }

    pub fn subtract_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::SubtractInteger);

        Term::builtin(arena, fun)
    }

    pub fn equals_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::EqualsInteger);

        Term::builtin(arena, fun)
    }

    pub fn less_than_equals_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LessThanEqualsInteger);

        Term::builtin(arena, fun)
    }

    pub fn less_than_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LessThanInteger);

        Term::builtin(arena, fun)
    }

    pub fn if_then_else(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::IfThenElse);

        Term::builtin(arena, fun)
    }

    pub fn append_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::AppendByteString);

        Term::builtin(arena, fun)
    }

    pub fn equals_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::EqualsByteString);

        Term::builtin(arena, fun)
    }

    pub fn cons_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::ConsByteString);

        Term::builtin(arena, fun)
    }

    pub fn slice_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::SliceByteString);

        Term::builtin(arena, fun)
    }

    pub fn length_of_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LengthOfByteString);

        Term::builtin(arena, fun)
    }

    pub fn index_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::IndexByteString);

        Term::builtin(arena, fun)
    }

    pub fn less_than_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LessThanByteString);

        Term::builtin(arena, fun)
    }

    pub fn less_than_equals_byte_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LessThanEqualsByteString);

        Term::builtin(arena, fun)
    }

    pub fn sha2_256(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::Sha2_256);

        Term::builtin(arena, fun)
    }

    pub fn sha3_256(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::Sha3_256);

        Term::builtin(arena, fun)
    }

    pub fn blake2b_256(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::Blake2b_256);

        Term::builtin(arena, fun)
    }

    pub fn keccak_256(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::Keccak_256);

        Term::builtin(arena, fun)
    }

    pub fn blake2b_224(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::Blake2b_224);

        Term::builtin(arena, fun)
    }

    pub fn verify_ed25519_signature(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::VerifyEd25519Signature);

        Term::builtin(arena, fun)
    }

    pub fn verify_ecdsa_secp256k1_signature(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::VerifyEcdsaSecp256k1Signature);

        Term::builtin(arena, fun)
    }

    pub fn verify_schnorr_secp256k1_signature(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::VerifySchnorrSecp256k1Signature);

        Term::builtin(arena, fun)
    }

    pub fn append_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::AppendString);

        Term::builtin(arena, fun)
    }

    pub fn equals_string(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::EqualsString);

        Term::builtin(arena, fun)
    }

    pub fn encode_utf8(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::EncodeUtf8);

        Term::builtin(arena, fun)
    }

    pub fn decode_utf8(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::DecodeUtf8);

        Term::builtin(arena, fun)
    }

    pub fn choose_unit(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::ChooseUnit);

        Term::builtin(arena, fun)
    }
}
