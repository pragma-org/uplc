mod builtin;
mod data;
pub mod decode;
mod zigzag;

use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};
use decode::{Ctx, Decoder, FlatDecodeError};

use crate::{
    constant::Constant,
    program::{Program, Version},
    term::Term,
};

const TERM_TAG_WIDTH: usize = 4;
const CONST_TAG_WIDTH: usize = 4;
const BUILTIN_TAG_WIDTH: usize = 7;

pub fn decode<'a>(arena: &'a Bump, bytes: &[u8]) -> Result<&'a Program<'a>, FlatDecodeError> {
    let mut decoder = Decoder::new(bytes);

    let major = decoder.word()?;
    let minor = decoder.word()?;
    let patch = decoder.word()?;

    let version = Version::new(arena, major, minor, patch);

    let mut ctx = Ctx { arena };

    let term = decode_term(&mut ctx, &mut decoder)?;

    decoder.filler()?;

    // TODO: probably should add a `finish()?` method that errors if bytes remain

    Ok(Program::new(arena, version, term))
}

fn decode_term<'a>(
    ctx: &mut Ctx<'a>,
    decoder: &mut Decoder<'_>,
) -> Result<&'a Term<'a>, FlatDecodeError> {
    let tag = decoder.bits8(TERM_TAG_WIDTH)?;

    match tag {
        // Var
        0 => Ok(Term::var(ctx.arena, decoder.word()?)),
        // Delay
        1 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.delay(ctx.arena))
        }
        // Lambda
        2 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.lambda(ctx.arena, 0))
        }
        // Apply
        3 => {
            let function = decode_term(ctx, decoder)?;
            let argument = decode_term(ctx, decoder)?;

            let term = function.apply(ctx.arena, argument);

            Ok(term)
        }
        // Constant
        4 => {
            let constant = decode_constant(ctx, decoder)?;

            Ok(Term::constant(ctx.arena, constant))
        }
        // Force
        5 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.force(ctx.arena))
        }
        // Error
        6 => Ok(Term::error(ctx.arena)),
        // Builtin
        7 => {
            let builtin_tag = decoder.bits8(BUILTIN_TAG_WIDTH)?;

            let function = builtin::try_from_tag(ctx.arena, builtin_tag)?;

            let term = Term::builtin(ctx.arena, function);

            Ok(term)
        }
        // Constr
        8 => {
            let tag = decoder.word()?;
            let fields = decoder.list_with(ctx, decode_term)?;

            let term = Term::constr(ctx.arena, tag, fields);

            Ok(term)
        }
        // Case
        9 => {
            let constr = decode_term(ctx, decoder)?;
            let branches = decoder.list_with(ctx, decode_term)?;

            Ok(Term::case(ctx.arena, constr, branches))
        }
        _ => Err(FlatDecodeError::UnknownTermConstructor(tag)),
    }
}

// BLS literals not supported
fn decode_constant<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    let tags = decode_constant_tags(ctx, d)?;

    match &tags.as_slice() {
        [0] => {
            let v = d.integer()?;

            Ok(Constant::integer_from(ctx.arena, v))
        }
        [1] => {
            let b = d.bytes(ctx.arena)?;

            Ok(Constant::byte_string(ctx.arena, b))
        }
        [2] => {
            let utf8_bytes = d.bytes(ctx.arena)?;

            let s = BumpString::from_utf8(utf8_bytes)
                .map_err(|e| FlatDecodeError::DecodeUtf8(e.utf8_error()))?;

            Ok(Constant::string(ctx.arena, s))
        }
        [3] => Ok(Constant::unit(ctx.arena)),
        [4] => {
            let v = d.bit()?;

            Ok(Constant::bool(ctx.arena, v))
        }
        [7, 5, rest @ ..] => todo!("list"),

        [7, 7, 6, rest @ ..] => todo!("pair"),

        [8] => {
            let cbor = d.bytes(ctx.arena)?;

            let data = minicbor::decode_with(&cbor, ctx)?;

            Ok(Constant::data(ctx.arena, data))
        }

        x => Err(FlatDecodeError::UnknownConstantConstructor(x.to_vec())),
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
