//! UPLC text-format parser.
//!
//! Parses UPLC source text into an arena-allocated AST. Variables are represented as
//! [`DeBruijn`] indices in the output.
//!
//! # Entry points
//!
//! - [`parse_program`] — parse a complete `(program 1.0.0 <term>)` expression
//! - [`parse_term`] — parse a single term
//! - [`parse_constant`] — parse a constant literal
//! - [`parse_data`] — parse a Plutus data value

use chumsky::{extra::SimpleState, prelude::*, ParseResult, Parser};

mod constant;
mod data;
mod program;
mod term;
mod typ;
mod types;
mod utils;
mod version;

use crate::{
    arena::Arena, binder::DeBruijn, constant::Constant, data::PlutusData, program::Program,
    term::Term,
};

/// Parses a complete UPLC program expression `(program <version> <term>)`.
pub fn parse_program<'a>(
    arena: &'a Arena,
    input: &'a str,
) -> ParseResult<&'a Program<'a, DeBruijn>, Rich<'a, char>> {
    let mut initial_state = SimpleState(types::State::new(arena));

    program::parser().parse_with_state(input, &mut initial_state)
}

/// Parses a single UPLC term.
pub fn parse_term<'a>(
    arena: &'a Arena,
    input: &'a str,
) -> ParseResult<&'a Term<'a, DeBruijn>, Rich<'a, char>> {
    let mut initial_state = SimpleState(types::State::new(arena));

    term::parser().parse_with_state(input, &mut initial_state)
}

/// Parses a UPLC constant literal.
pub fn parse_constant<'a>(
    arena: &'a Arena,
    input: &'a str,
) -> ParseResult<&'a Constant<'a>, Rich<'a, char>> {
    let mut initial_state = SimpleState(types::State::new(arena));

    constant::parser().parse_with_state(input, &mut initial_state)
}

/// Parses a Plutus data value.
pub fn parse_data<'a>(
    arena: &'a Arena,
    input: &'a str,
) -> ParseResult<&'a PlutusData<'a>, Rich<'a, char>> {
    let mut initial_state = SimpleState(types::State::new(arena));

    data::parser().parse_with_state(input, &mut initial_state)
}
