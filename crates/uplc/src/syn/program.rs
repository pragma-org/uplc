use chumsky::{prelude::*, Parser};

use crate::program::Program;

use super::{term, types::Extra, version};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a mut Program<'a>, Extra<'a>> {
    text::keyword("program")
        .ignore_then(version::parser())
        .then(term::parser())
        .delimited_by(just('('), just(')'))
        .map_with(|(version, term), e| {
            let state = e.state();

            Program::new(state.arena, version, term)
        })
}
