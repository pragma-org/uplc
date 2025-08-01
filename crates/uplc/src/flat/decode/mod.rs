mod decoder;
mod error;

pub use decoder::Ctx;
pub use decoder::Decoder;
pub use error::FlatDecodeError;

use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::binder::Binder;
use crate::typ::Type;
use crate::{
    constant::Constant,
    program::{Program, Version},
    term::Term,
};

use super::tag;
use super::{
    builtin,
    tag::{BUILTIN_TAG_WIDTH, CONST_TAG_WIDTH, TERM_TAG_WIDTH},
};

pub fn decode<'a, V>(arena: &'a Bump, bytes: &[u8]) -> Result<&'a Program<'a, V>, FlatDecodeError>
where
    V: Binder<'a>,
{
    let mut decoder = Decoder::new(bytes);

    let major = decoder.word()?;
    let minor = decoder.word()?;
    let patch = decoder.word()?;

    let version = Version::new(arena, major, minor, patch);

    let mut ctx = Ctx { arena };

    let term = decode_term(&mut ctx, &mut decoder)?;

    decoder.filler()?;

    Ok(Program::new(arena, version, term))
}

fn decode_term<'a, V>(
    ctx: &mut Ctx<'a>,
    decoder: &mut Decoder<'_>,
) -> Result<&'a Term<'a, V>, FlatDecodeError>
where
    V: Binder<'a>,
{
    let tag = decoder.bits8(TERM_TAG_WIDTH)?;

    match tag {
        // Var
        tag::VAR => Ok(Term::var(ctx.arena, V::var_decode(ctx.arena, decoder)?)),
        // Delay
        tag::DELAY => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.delay(ctx.arena))
        }
        // Lambda
        tag::LAMBDA => {
            let param = V::parameter_decode(ctx.arena, decoder)?;

            let term = decode_term(ctx, decoder)?;

            Ok(term.lambda(ctx.arena, param))
        }
        // Apply
        tag::APPLY => {
            let function = decode_term(ctx, decoder)?;
            let argument = decode_term(ctx, decoder)?;

            let term = function.apply(ctx.arena, argument);

            Ok(term)
        }
        // Constant
        tag::CONSTANT => {
            let constant = decode_constant(ctx, decoder)?;

            Ok(Term::constant(ctx.arena, constant))
        }
        // Force
        tag::FORCE => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.force(ctx.arena))
        }
        // Error
        tag::ERROR => Ok(Term::error(ctx.arena)),
        // Builtin
        tag::BUILTIN => {
            let builtin_tag = decoder.bits8(BUILTIN_TAG_WIDTH)?;

            let function = builtin::try_from_tag(ctx.arena, builtin_tag)?;

            let term = Term::builtin(ctx.arena, function);

            Ok(term)
        }
        // Constr
        tag::CONSTR => {
            let tag = decoder.word()?;
            let fields = decoder.list_with(ctx, decode_term)?;
            let fields = ctx.arena.alloc(fields);

            let term = Term::constr(ctx.arena, tag, fields);

            Ok(term)
        }
        // Case
        tag::CASE => {
            let constr = decode_term(ctx, decoder)?;
            let branches = decoder.list_with(ctx, decode_term)?;
            let branches = ctx.arena.alloc(branches);

            Ok(Term::case(ctx.arena, constr, branches))
        }
        _ => Err(FlatDecodeError::UnknownTermConstructor(tag)),
    }
}

fn decode_type<'a>(ctx: &mut Ctx<'a>, d: &mut Decoder) -> Result<&'a Type<'a>, FlatDecodeError> {
    let tag = decode_constant_tags(ctx, d)?;

    match &tag.as_slice() {
        [tag::INTEGER] => Ok(Type::integer(ctx.arena)),
        [tag::BYTE_STRING] => Ok(Type::byte_string(ctx.arena)),
        [tag::STRING] => Ok(Type::string(ctx.arena)),
        [tag::UNIT] => Ok(Type::unit(ctx.arena)),
        [tag::BOOL] => Ok(Type::bool(ctx.arena)),
        [tag::PROTO_LIST_ONE, tag::PROTO_LIST_TWO] => {
            let sub_typ = decode_type(ctx, d)?;
            Ok(Type::list(ctx.arena, sub_typ))
        }
        [tag::PROTO_LIST_ONE, tag::PROTO_LIST_TWO, tag::DATA] => {
            Ok(Type::list(ctx.arena, &Type::Data))
        }
        [tag::PROTO_PAIR_ONE, tag::PROTO_PAIR_TWO, tag::PROTO_PAIR_THREE] => {
            let sub_typ1 = decode_type(ctx, d)?;
            let sub_typ2 = decode_type(ctx, d)?;
            Ok(Type::pair(ctx.arena, sub_typ1, sub_typ2))
        }
        [tag::DATA] => Ok(Type::data(ctx.arena)),
        [] => Err(FlatDecodeError::MissingTypeTag),
        x => Err(FlatDecodeError::UnknownTypeTags(x.to_vec())),
    }
}

// BLS literals not supported
fn decode_constant<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    let ty = decode_type(ctx, d)?;

    match ty {
        Type::Integer => {
            let v = d.integer()?;
            let v = ctx.arena.alloc(v);

            Ok(Constant::integer(ctx.arena, v))
        }
        Type::ByteString => {
            let b = d.bytes(ctx.arena)?;
            let b = ctx.arena.alloc(b);

            Ok(Constant::byte_string(ctx.arena, b))
        }
        Type::Bool => {
            let v = d.bit()?;

            Ok(Constant::bool(ctx.arena, v))
        }
        Type::String => {
            let s = d.utf8(ctx.arena)?;
            let s = ctx.arena.alloc(s);

            Ok(Constant::string(ctx.arena, s))
        }
        Type::Unit => Ok(Constant::unit(ctx.arena)),
        Type::List(sub_typ) => {
            let fields = d.list_with(ctx, |ctx, d| decode_constant(ctx, d))?;
            let fields = ctx.arena.alloc(fields);

            Ok(Constant::proto_list(ctx.arena, sub_typ, fields))
        }
        Type::Pair(sub_typ1, sub_typ2) => {
            let fst = decode_constant(ctx, d)?;
            let snd = decode_constant(ctx, d)?;

            Ok(Constant::proto_pair(
                ctx.arena, sub_typ1, sub_typ2, fst, snd,
            ))
        }
        Type::Data => {
            let cbor = d.bytes(ctx.arena)?;
            let data = minicbor::decode_with(&cbor, ctx)?;

            Ok(Constant::data(ctx.arena, data))
        }
        Type::Bls12_381G1Element => Err(FlatDecodeError::BlsTypeNotSupported),
        Type::Bls12_381G2Element => Err(FlatDecodeError::BlsTypeNotSupported),
        Type::Bls12_381MlResult => Err(FlatDecodeError::BlsTypeNotSupported),
    }
}

fn decode_constant_tags<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
    d.list_with(ctx, |_arena, d| decode_constant_tag(d))
}

fn decode_constant_tag(d: &mut Decoder) -> Result<u8, FlatDecodeError> {
    d.bits8(CONST_TAG_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binder::DeBruijn;
    use hex;
    use num::BigInt;

    #[test]
    fn decode_program_1() {
        // (program 1.1.0
        //   [
        //     [
        //       (builtin addInteger)
        //       (con integer 1)
        //     ]
        //     [ (force (force (builtin fstPair)))
        //       [ (builtin unConstrData)
        //         (con data (Constr 128 [I 0, I 1]))
        //       ]
        //     ]
        //   ])
        let bytes = hex::decode("0101003370090011aab9d375498109d8668218809f0001ff0001").unwrap();
        let arena = Bump::new();
        let program: Result<&Program<DeBruijn>, _> = decode(&arena, &bytes);
        match program {
            Ok(program) => {
                let eval_result = program.eval(&arena);
                let term = eval_result.term.unwrap();
                assert_eq!(term, &Term::Constant(&Constant::Integer(&BigInt::from(129))));
            },
            Err(e) => {
                assert!(false);
            }
        }
    }
}
