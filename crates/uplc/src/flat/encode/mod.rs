mod encoder;
mod error;

pub use encoder::Encoder;
pub use error::FlatEncodeError;

use crate::{binder::Binder, constant::Constant, program::Program, term::Term};

use super::tag;

pub fn encode<'a, V>(program: &'a Program<'a, V>) -> Result<Vec<u8>, FlatEncodeError>
where
    V: Binder<'a>,
{
    let mut encoder = Encoder::default();

    encoder
        .word(program.version.major())
        .word(program.version.minor())
        .word(program.version.patch());

    encode_term(&mut encoder, program.term)?;

    encoder.filler();

    Ok(encoder.buffer)
}

fn encode_term<'a, V>(encoder: &mut Encoder, term: &'a Term<'a, V>) -> Result<(), FlatEncodeError>
where
    V: Binder<'a>,
{
    match term {
        Term::Var(name) => {
            encode_term_tag(encoder, tag::VAR)?;

            name.var_encode(encoder)?;
        }
        Term::Lambda { parameter, body } => {
            encode_term_tag(encoder, tag::LAMBDA)?;

            parameter.parameter_encode(encoder)?;

            encode_term(encoder, body)?;
        }
        Term::Apply { function, argument } => {
            encode_term_tag(encoder, tag::APPLY)?;

            encode_term(encoder, function)?;

            encode_term(encoder, argument)?;
        }
        Term::Delay(body) => {
            encode_term_tag(encoder, tag::DELAY)?;

            encode_term(encoder, body)?;
        }
        Term::Force(body) => {
            encode_term_tag(encoder, tag::FORCE)?;

            encode_term(encoder, body)?;
        }
        Term::Case { constr, branches } => {
            encode_term_tag(encoder, tag::CASE)?;

            encode_term(encoder, constr)?;

            encoder.list_with(branches, |e, t| encode_term(e, t))?;
        }
        Term::Constr { tag, fields } => {
            encode_term_tag(encoder, tag::CONSTR)?;

            encoder.word(*tag);

            encoder.list_with(fields, |e, t| encode_term(e, t))?;
        }
        Term::Constant(c) => {
            encode_term_tag(encoder, tag::CONSTANT)?;

            encode_constant(encoder, c)?;
        }
        Term::Builtin(b) => {
            encode_term_tag(encoder, tag::BUILTIN)?;

            encoder.bits(tag::BUILTIN_TAG_WIDTH as i64, **b as u8);
        }
        Term::Error => {
            encode_term_tag(encoder, tag::ERROR)?;
        }
    }

    Ok(())
}

fn encode_constant<'a>(e: &mut Encoder, constant: &'a Constant<'a>) -> Result<(), FlatEncodeError> {
    match constant {
        Constant::Integer(i) => {
            e.list_with(&[0], encode_constant_tag)?;

            e.integer(i);
        }
        Constant::ByteString(b) => {
            e.list_with(&[1], encode_constant_tag)?;

            e.bytes(b)?;
        }
        Constant::String(s) => {
            e.list_with(&[2], encode_constant_tag)?;

            e.utf8(s)?;
        }
        Constant::Unit => {
            e.list_with(&[3], encode_constant_tag)?;
        }
        Constant::Boolean(b) => {
            e.list_with(&[4], encode_constant_tag)?;

            e.bool(*b);
        }
        Constant::Data(_) => {}
        Constant::ProtoList(_, _) => todo!(),
        Constant::ProtoPair(_, _, _, _) => todo!(),
        Constant::Bls12_381G1Element(_) => todo!(),
        Constant::Bls12_381G2Element(_) => todo!(),
        Constant::Bls12_381MlResult(_) => todo!(),
    }

    Ok(())
}

fn encode_term_tag(e: &mut Encoder, tag: u8) -> Result<(), FlatEncodeError> {
    safe_encode_bits(e, tag::TERM_TAG_WIDTH, tag)
}

fn encode_constant_tag(e: &mut Encoder, tag: &u8) -> Result<(), FlatEncodeError> {
    safe_encode_bits(e, tag::CONST_TAG_WIDTH, *tag)
}

fn safe_encode_bits(e: &mut Encoder, num_bits: usize, byte: u8) -> Result<(), FlatEncodeError> {
    if 2_u8.pow(num_bits as u32) <= byte {
        Err(FlatEncodeError::Overflow { byte, num_bits })
    } else {
        e.bits(num_bits as i64, byte);

        Ok(())
    }
}
