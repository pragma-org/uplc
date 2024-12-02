mod encoder;
mod error;

pub use encoder::Encoder;
pub use error::FlatEncodeError;

use crate::{binder::Binder, program::Program, term::Term};

use super::tag::{self, BUILTIN_TAG_WIDTH, TERM_TAG_WIDTH};

pub fn encode<'a, V>(program: &'a Program<'a, V>) -> Result<Vec<u8>, FlatEncodeError>
where
    V: Binder,
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
    V: Binder,
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
        Term::Constant(c) => todo!(),
        Term::Builtin(b) => {
            encode_term_tag(encoder, tag::BUILTIN)?;

            encoder.bits(BUILTIN_TAG_WIDTH as i64, **b as u8);
        }
        Term::Error => {
            encode_term_tag(encoder, tag::ERROR)?;
        }
    }

    Ok(())
}

fn encode_term_tag(e: &mut Encoder, tag: u8) -> Result<(), FlatEncodeError> {
    safe_encode_bits(e, TERM_TAG_WIDTH, tag)
}

fn safe_encode_bits(e: &mut Encoder, num_bits: usize, byte: u8) -> Result<(), FlatEncodeError> {
    if 2_u8.pow(num_bits as u32) <= byte {
        Err(FlatEncodeError::Overflow { byte, num_bits })
    } else {
        e.bits(num_bits as i64, byte);

        Ok(())
    }
}
