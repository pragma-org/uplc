pub mod decode;

use bumpalo::{collections::Vec as BumpVec, Bump};
use decode::{Decoder, FlatDecodeError};

use crate::{
    constant::Constant,
    program::{Program, Version},
    term::Term,
};

pub fn decode<'a>(arena: &'a Bump, bytes: &[u8]) -> Result<&'a Program<'a>, FlatDecodeError> {
    let mut decoder = Decoder::new(bytes);

    let major = decoder.word()?;
    let minor = decoder.word()?;
    let patch = decoder.word()?;

    let version = Version::new(arena, major, minor, patch);

    let term = decode_term(arena, &mut decoder)?;

    decoder.filler()?;

    // TODO: probably should add a `finish()?` method that errors if bytes remain

    Ok(Program::new(arena, version, term))
}

const TERM_TAG_WIDTH: usize = 4;

fn decode_term<'a>(
    arena: &'a Bump,
    decoder: &mut Decoder<'_>,
) -> Result<&'a Term<'a>, FlatDecodeError> {
    let tag = decoder.bits8(TERM_TAG_WIDTH)?;

    match tag {
        // Var
        0 => Ok(Term::var(arena, decoder.word()?)),
        // Delay
        1 => {
            let term = decode_term(arena, decoder)?;

            Ok(term.delay(arena))
        }
        // Lambda
        2 => {
            let term = decode_term(arena, decoder)?;

            Ok(term.lambda(arena, 0))
        }
        // Apply
        3 => {
            let function = decode_term(arena, decoder)?;
            let argument = decode_term(arena, decoder)?;

            let term = function.apply(arena, argument);

            Ok(term)
        }
        // Constant
        4 => {
            let constant = decode_constant(arena, decoder)?;

            Ok(Term::constant(arena, constant))
        }
        // Force
        5 => {
            let term = decode_term(arena, decoder)?;

            Ok(term.force(arena))
        }
        // Error
        6 => Ok(Term::error(arena)),
        // Builtin
        7 => {
            todo!("decode builtin")
        }
        // Constr
        8 => {
            let tag = decoder.word()?;
            let fields = decoder.list_with(arena, decode_term)?;

            let term = Term::constr(arena, tag, fields);

            Ok(term)
        }
        // Case
        9 => {
            let constr = decode_term(arena, decoder)?;
            let branches = decoder.list_with(arena, decode_term)?;

            Ok(Term::case(arena, constr, branches))
        }
        _ => Err(FlatDecodeError::UnknownTermConstructor(tag)),
    }
}

// BLS literals not supported
fn decode_constant<'a>(
    arena: &'a Bump,
    d: &mut Decoder,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    let tags = decode_constant_tags(arena, d)?;

    match &tags.as_slice() {
        [0] => todo!("integer"),
        [1] => todo!("bytestring"),
        [2] => todo!("string"),
        [3] => Ok(Constant::unit(arena)),
        [4] => {
            let v = d.bit()?;

            Ok(Constant::bool(arena, v))
        }
        [7, 5, rest @ ..] => todo!("list"),

        [7, 7, 6, rest @ ..] => todo!("pair"),

        [8] => todo!("data"),

        x => Err(FlatDecodeError::UnknownConstantConstructor(x.to_vec())),
    }
}

fn decode_constant_tags<'a>(
    arena: &'a Bump,
    d: &mut Decoder,
) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
    d.list_with(arena, |_arena, d| decode_constant_tag(d))
}

const CONST_TAG_WIDTH: usize = 4;

pub fn decode_constant_tag(d: &mut Decoder) -> Result<u8, FlatDecodeError> {
    d.bits8(CONST_TAG_WIDTH)
}
