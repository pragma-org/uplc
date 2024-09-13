use chumsky::{prelude::*, Parser};

use crate::term::Term;

use super::types::{Extra, MapExtra};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a Term<'a>, Extra<'a>> {
    recursive(|t: Recursive<dyn Parser<'_, &str, &Term<'_>, Extra<'a>>>| {
        choice((
            // Var
            text::ident().validate(|v, e: &mut MapExtra<'a, '_>, emit| {
                let state = e.state();

                let position = state.env.iter().rev().position(|&x| x == v);

                if position.is_none() {
                    // TODO: return OpenTermError
                    // emit(Simple);
                }

                let debruijn_index = state.env.len() - position.unwrap_or_default();

                Term::var(state.arena, debruijn_index)
            }),
            // Delay
            text::keyword("delay")
                .ignore_then(t.clone())
                .delimited_by(just('('), just(')'))
                .map_with(|term, e| {
                    let state = e.state();

                    term.delay(state.arena)
                }),
            // Force
            text::keyword("force")
                .ignore_then(t.clone())
                .delimited_by(just('('), just(')'))
                .map_with(|term, e| {
                    let state = e.state();

                    term.force(state.arena)
                }),
            // Lambda
            text::keyword("lambda")
                .ignore_then(text::ident())
                .then(t.clone())
                .delimited_by(just('('), just(')'))
                .map_with(|(v, term), e| {
                    let state = e.state();

                    state.env.push(v);

                    let lambda = term.lambda(state.arena, 0);

                    state.env.pop();

                    lambda
                }),
            // Apply
            t.clone()
                .foldl_with(t.repeated().at_least(1), |a, b, e| {
                    let state = e.state();

                    a.apply(state.arena, b)
                })
                .delimited_by(just('['), just(']')),
            // Error
            text::keyword("error")
                .ignored()
                .delimited_by(just('('), just(')'))
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Term::error(state.arena)
                }),
        ))
        .boxed()
    })
}
