use bumpalo::Bump;
use chumsky::{prelude::*, Parser};

use crate::term::Term;

use super::{
    constant,
    types::{Extra, MapExtra},
};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a Term<'a>, Extra<'a>> {
    recursive(|term| {
        choice((
            // Var
            text::ident()
                .padded()
                .validate(|v, e: &mut MapExtra<'a, '_>, emitter| {
                    let state = e.state();

                    let position = state.env.iter().rev().position(|&x| x == v);

                    if position.is_none() {
                        let placeholder = Term::var(state.arena, 0);

                        emitter.emit(Rich::custom(e.span(), "open term"));

                        placeholder
                    } else {
                        let debruijn_index = state.env.len() - position.unwrap_or_default();

                        Term::var(state.arena, debruijn_index)
                    }
                }),
            // Delay
            text::keyword("delay")
                .padded()
                .ignore_then(term.clone().padded())
                .delimited_by(just('('), just(')'))
                .map_with(|term: &Term<'_>, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    term.delay(state.arena)
                }),
            // Force
            text::keyword("force")
                .padded()
                .ignore_then(term.clone().padded())
                .delimited_by(just('('), just(')'))
                .map_with(|term, e| {
                    let state = e.state();

                    term.force(state.arena)
                }),
            // Lambda
            text::keyword("lam")
                .padded()
                .ignore_then(text::ident().padded())
                .map_with(|v, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    state.env.push(v);

                    0
                })
                .then(term.clone().padded())
                .delimited_by(just('('), just(')'))
                .map_with(|(v, term), e| {
                    let state = e.state();

                    state.env.pop();

                    term.lambda(state.arena, v)
                }),
            // Apply
            term.clone()
                .padded()
                .foldl_with(term.padded().repeated().at_least(1), |a, b, e| {
                    let state = e.state();

                    a.apply(state.arena, b)
                })
                .delimited_by(just('['), just(']')),
            // Constant
            constant::parser().map_with(|c, e: &mut MapExtra<'a, '_>| {
                let state = e.state();

                Term::constant(state.arena, c)
            }),
            // Builtin
            text::keyword("builtin")
                .padded()
                .ignore_then(text::ident().padded())
                .delimited_by(just('('), just(')'))
                .validate(|v, e: &mut MapExtra<'a, '_>, emitter| {
                    let state = e.state();

                    if let Some(builtin) = builtin_from_str(state.arena, v) {
                        builtin
                    } else {
                        let builtin = Term::error(state.arena);

                        emitter.emit(Rich::custom(e.span(), "unknown builtin"));

                        builtin
                    }
                }),
            // Error
            text::keyword("error")
                .padded()
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

pub fn builtin_from_str<'a>(arena: &'a Bump, name: &str) -> Option<&'a Term<'a>> {
    match name {
        "addInteger" => Some(Term::add_integer(arena)),
        "multiplyInteger" => Some(Term::multiply_integer(arena)),
        "equalsInteger" => Some(Term::equals_integer(arena)),
        "lessThanEqualsInteger" => Some(Term::less_than_equals_integer(arena)),
        "lessThanInteger" => Some(Term::less_than_integer(arena)),
        "subtractInteger" => Some(Term::subtract_integer(arena)),
        "ifThenElse" => Some(Term::if_then_else(arena)),
        "appendByteString" => Some(Term::append_byte_string(arena)),
        "equalsByteString" => Some(Term::equals_byte_string(arena)),
        "lengthOfByteString" => Some(Term::length_of_byte_string(arena)),
        "indexByteString" => Some(Term::index_byte_string(arena)),
        "lessThanByteString" => Some(Term::less_than_byte_string(arena)),
        "lessThanEqualsByteString" => Some(Term::less_than_equals_byte_string(arena)),
        "blake2b_256" => Some(Term::blake2b_256(arena)),
        _ => None,
    }
}
