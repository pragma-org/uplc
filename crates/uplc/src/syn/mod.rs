use bumpalo::Bump;
use chumsky::{prelude::*, ParseResult, Parser};

mod data;
mod program;
mod term;
mod types;
mod utils;
mod version;

use crate::{program::Program, term::Term};

pub fn parse_program<'a>(
    arena: &'a Bump,
    input: &'a str,
) -> ParseResult<&'a mut Program<'a>, Simple<'a, char>> {
    let mut initial_state = types::State::new(arena);

    program::parser().parse_with_state(input, &mut initial_state)
}

pub fn parse_term<'a>(
    arena: &'a Bump,
    input: &'a str,
) -> ParseResult<&'a Term<'a>, Simple<'a, char>> {
    let mut initial_state = types::State::new(arena);

    term::parser().parse_with_state(input, &mut initial_state)
}
