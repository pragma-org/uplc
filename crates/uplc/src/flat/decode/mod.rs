mod decoder;
mod error;

use std::collections::VecDeque;

pub use decoder::Ctx;
pub use decoder::Decoder;
pub use error::FlatDecodeError;

use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};

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

fn tags_to_type<'a>(
    ctx: &mut Ctx<'a>,
    tags: &mut VecDeque<u8>,
) -> Result<&'a Type<'a>, FlatDecodeError> {
    match tags.pop_front() {
        Some(tag::INTEGER) => Ok(Type::integer(ctx.arena)),
        Some(tag::BYTE_STRING) => Ok(Type::byte_string(ctx.arena)),
        Some(tag::STRING) => Ok(Type::string(ctx.arena)),
        Some(tag::UNIT) => Ok(Type::unit(ctx.arena)),
        Some(tag::BOOL) => Ok(Type::bool(ctx.arena)),
        Some(tag::DATA) => Ok(Type::data(ctx.arena)),
        Some(tag::PROTO_LIST_ONE) => match tags.pop_front() {
            Some(tag::PROTO_LIST_TWO) => {
                let sub_typ = tags_to_type(ctx, tags)?;
                Ok(Type::list(ctx.arena, sub_typ))
            }
            Some(tag::PROTO_PAIR_TWO) => match tags.pop_front() {
                Some(tag::PROTO_PAIR_THREE) => {
                    let type_a = tags_to_type(ctx, tags)?;
                    let type_b = tags_to_type(ctx, tags)?;
                    Ok(Type::pair(ctx.arena, type_a, type_b))
                }
                _ => {
                    tags.push_front(tag::PROTO_LIST_ONE);
                    tags.push_front(tag::PROTO_PAIR_TWO);
                    Err(FlatDecodeError::UnknownTypeTags(tags.clone().into()))
                }
            },
            _ => {
                tags.push_front(tag::PROTO_LIST_ONE);
                Err(FlatDecodeError::UnknownTypeTags(tags.clone().into()))
            }
        },

        None => Err(FlatDecodeError::MissingTypeTag),
        Some(x) => Err(FlatDecodeError::UnknownTypeTags(vec![x])),
    }
}

fn decode_type<'a>(ctx: &mut Ctx<'a>, d: &mut Decoder) -> Result<&'a Type<'a>, FlatDecodeError> {
    let tags = decode_constant_tags(ctx, d)?;

    let mut tags = VecDeque::from(tags.iter().copied().collect::<Vec<_>>());

    tags_to_type(ctx, &mut tags)
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
