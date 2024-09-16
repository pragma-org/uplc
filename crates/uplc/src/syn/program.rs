use chumsky::{prelude::*, Parser};

use crate::program::Program;

use super::{term, types::Extra, version};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a mut Program<'a>, Extra<'a>> {
    text::keyword("program")
        .padded()
        .ignore_then(version::parser().padded())
        .then(term::parser().padded())
        .delimited_by(just('('), just(')'))
        .padded()
        .then_ignore(end())
        .map_with(|(version, term), e| {
            let state = e.state();

            Program::new(state.arena, version, term)
        })
}
