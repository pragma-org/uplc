mod decoder;
mod error;

pub use decoder::Ctx;
pub use decoder::Decoder;
pub use error::FlatDecodeError;

use bumpalo::collections::Vec as BumpVec;

use crate::arena::Arena;
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

pub fn decode<'a, V>(arena: &'a Arena, bytes: &[u8]) -> Result<&'a Program<'a, V>, FlatDecodeError>
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

fn type_from_tags<'a>(
    ctx: &Ctx<'a>,
    tags: &[u8],
) -> Result<(&'a Type<'a>, usize), FlatDecodeError> {
    match tags {
        [tag::INTEGER, ..] => Ok((Type::integer(ctx.arena), 1)),
        [tag::BYTE_STRING, ..] => Ok((Type::byte_string(ctx.arena), 1)),
        [tag::STRING, ..] => Ok((Type::string(ctx.arena), 1)),
        [tag::UNIT, ..] => Ok((Type::unit(ctx.arena), 1)),
        [tag::BOOL, ..] => Ok((Type::bool(ctx.arena), 1)),
        [tag::DATA, ..] => Ok((Type::data(ctx.arena), 1)),
        [tag::PROTO_LIST_ONE, tag::PROTO_LIST_TWO, rest @ ..] => {
            let (sub_typ, consumed) = type_from_tags(ctx, rest)?;
            Ok((Type::list(ctx.arena, sub_typ), 2 + consumed))
        }
        [tag::PROTO_ARRAY_ONE, tag::PROTO_ARRAY_TWO, rest @ ..] => {
            let (sub_typ, consumed) = type_from_tags(ctx, rest)?;
            Ok((Type::array(ctx.arena, sub_typ), 2 + consumed))
        }
        [tag::PROTO_PAIR_ONE, tag::PROTO_PAIR_TWO, tag::PROTO_PAIR_THREE, rest @ ..] => {
            let (sub_typ1, consumed1) = type_from_tags(ctx, rest)?;
            let rest2 = &rest[consumed1..];
            let (sub_typ2, consumed2) = type_from_tags(ctx, rest2)?;

            Ok((
                Type::pair(ctx.arena, sub_typ1, sub_typ2),
                3 + consumed1 + consumed2,
            ))
        }
        [] => Err(FlatDecodeError::MissingTypeTag),
        x => Err(FlatDecodeError::UnknownTypeTags(x.to_vec())),
    }
}

// BLS literals not supported
fn decode_constant<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    let tags = decode_constant_tags(ctx, d)?;
    let (ty, _) = type_from_tags(ctx, tags.as_slice())?;

    match ty {
        Type::Integer => {
            let v = d.integer()?;
            let v = ctx.arena.alloc_integer(v);

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
            let fields = d.list_with(ctx, |ctx, d| decode_constant_with_type(ctx, d, sub_typ))?;
            let fields = ctx.arena.alloc(fields);

            Ok(Constant::proto_list(ctx.arena, sub_typ, fields))
        }

        Type::Array(sub_typ) => {
            let fields = d.list_with(ctx, |ctx, d| decode_constant_with_type(ctx, d, sub_typ))?;
            let fields = ctx.arena.alloc(fields);
            Ok(Constant::proto_array(ctx.arena, sub_typ, fields))
        }
        Type::Pair(sub_typ1, sub_typ2) => {
            let fst = decode_constant_with_type(ctx, d, sub_typ1)?;
            let snd = decode_constant_with_type(ctx, d, sub_typ2)?;

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

// BLS literals not supported
fn decode_constant_with_type<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
    ty: &Type<'a>,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    match ty {
        Type::Integer => {
            let v = d.integer()?;
            let v = ctx.arena.alloc_integer(v);

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
            let fields = d.list_with(ctx, |ctx, d| decode_constant_with_type(ctx, d, sub_typ))?;
            let fields = ctx.arena.alloc(fields);

            Ok(Constant::proto_list(ctx.arena, sub_typ, fields))
        }
        Type::Array(sub_typ) => {
            let fields = d.list_with(ctx, |ctx, d| decode_constant_with_type(ctx, d, sub_typ))?;
            let fields = ctx.arena.alloc(fields);
            Ok(Constant::proto_array(ctx.arena, sub_typ, fields))
        }
        Type::Pair(sub_typ1, sub_typ2) => {
            let fst = decode_constant_with_type(ctx, d, sub_typ1)?;
            let snd = decode_constant_with_type(ctx, d, sub_typ2)?;

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
    use crate::{arena::Arena, binder::DeBruijn};
    use hex;
    use num::BigInt;

    #[test]
    fn decode_program_big_constr_tag() {
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
        let arena = Arena::new();
        let program: Result<&Program<DeBruijn>, _> = decode(&arena, &bytes);
        match program {
            Ok(program) => {
                let eval_result = program.eval(&arena);
                let term = eval_result.term.unwrap();
                assert_eq!(
                    term,
                    &Term::Constant(&Constant::Integer(&BigInt::from(129)))
                );
            }
            Err(_) => {
                panic!();
            }
        }
    }

    #[test]
    fn decode_program_bigint() {
        // (program 1.1.0
        //   [
        //     [
        //       (builtin addInteger)
        //       (con integer 1)
        //     ]
        //     [ (builtin unIData)
        //       [ (force (builtin headList))
        //         [ (force (force (builtin sndPair)))
        //           [ (builtin unConstrData)
        //             (con data (Constr 0 [I 999999999999999999999999999]))
        //           ]
        //         ]
        //       ]
        //     ]
        //   ])
        let bytes = hex::decode(
            "0101003370090011bad357426aae78dd526112d8799fc24c033b2e3c9fd0803ce7ffffffff0001",
        )
        .unwrap();
        let arena = Arena::new();
        let program: Result<&Program<DeBruijn>, _> = decode(&arena, &bytes);
        match program {
            Ok(program) => {
                let eval_result = program.eval(&arena);
                let term = eval_result.term.unwrap();
                assert_eq!(
                    term,
                    &Term::Constant(&Constant::Integer(&BigInt::from(
                        1_000_000_000_000_000_000_000_000_000i128
                    )))
                );
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    #[test]
    fn decode_program_list() {
        // (program 1.1.0
        //   [
        //     [
        //       (builtin multiplyInteger)
        //       (con integer 2)
        //     ]
        //     [ (builtin unIData)
        //       [ (force (builtin headList))
        //         [ (force (builtin tailList))
        //           [ (builtin unListData)
        //             (con data (List [I 7, I 14]))
        //           ]
        //         ]
        //       ]
        //     ]
        //   ])
        let bytes = hex::decode("0101003370490021bad357426ae88dd62601049f070eff0001").unwrap();
        let arena = Arena::new();
        let program: Result<&Program<DeBruijn>, _> = decode(&arena, &bytes);
        match program {
            Ok(program) => {
                let eval_result = program.eval(&arena);
                let term = eval_result.term.unwrap();
                assert_eq!(term, &Term::Constant(&Constant::Integer(&BigInt::from(28))));
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
