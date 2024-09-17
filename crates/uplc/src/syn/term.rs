use bumpalo::collections::{String as BumpString, Vec as BumpVec};
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
                .map_with(|v, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    state.env.push(v);

                    0
                })
                .then(t.clone())
                .delimited_by(just('('), just(')'))
                .map_with(|(v, term), e| {
                    let state = e.state();

                    state.env.pop();

                    term.lambda(state.arena, v)
                }),
            // Apply
            t.clone()
                .foldl_with(t.repeated().at_least(1), |a, b, e| {
                    let state = e.state();

                    a.apply(state.arena, b)
                })
                .delimited_by(just('['), just(']')),
            // constant
            text::keyword("con")
                .padded()
                .ignore_then(choice((
                    // integer
                    text::keyword("integer")
                        .padded()
                        .ignore_then(text::int(10).padded())
                        .map_with(|v, e: &mut MapExtra<'a, '_>| {
                            let state = e.state();

                            Term::integer_from(state.arena, v.parse().unwrap())
                        }),
                    // bytestring
                    text::keyword("bytestring")
                        .padded()
                        .ignore_then(just('#').ignore_then(hex_bytes()).padded())
                        .map_with(|v, e: &mut MapExtra<'a, '_>| {
                            let state = e.state();

                            let bytes = BumpVec::from_iter_in(v, state.arena);

                            Term::bytestring(state.arena, bytes)
                        }),
                    // string any utf8 encoded string surrounded in double quotes
                    text::keyword("string")
                        .padded()
                        .ignore_then(
                            just('"')
                                .ignore_then(string_content())
                                .then_ignore(just('"'))
                                .padded(),
                        )
                        .map_with(|v, e: &mut MapExtra<'a, '_>| {
                            let state = e.state();

                            let string = BumpString::from_str_in(&v, state.arena);

                            Term::string(state.arena, string)
                        }),
                    // bool
                    text::keyword("bool")
                        .padded()
                        .ignore_then(choice((just("False"), just("True"))).padded())
                        .map_with(|v, e: &mut MapExtra<'a, '_>| {
                            let state = e.state();

                            Term::bool(state.arena, v == "True")
                        }),
                    // unit
                    text::keyword("unit")
                        .padded()
                        .ignore_then(just("()").padded())
                        .ignored()
                        .map_with(|_v, e: &mut MapExtra<'a, '_>| {
                            let state = e.state();

                            Term::unit(state.arena)
                        }),
                )))
                .delimited_by(just('('), just(')')),
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

fn hex_digit<'a>() -> impl Parser<'a, &'a str, u8, Extra<'a>> {
    one_of("0123456789abcdefABCDEF").map(|c: char| c.to_digit(16).unwrap() as u8)
}

fn hex_bytes<'a>() -> impl Parser<'a, &'a str, Vec<u8>, Extra<'a>> {
    hex_digit()
        .then(hex_digit())
        .map(|(high, low)| (high << 4) | low)
        .repeated()
        .collect()
}

fn string_content<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> {
    let escape_sequence = just('\\').ignore_then(choice((
        just('a').to('\u{07}'),
        just('b').to('\u{08}'),
        just('f').to('\u{0C}'),
        just('n').to('\n'),
        just('r').to('\r'),
        just('t').to('\t'),
        just('v').to('\u{0B}'),
        just('\\'),
        just('"'),
        just('\''),
        just('&'),
        just('x').ignore_then(
            any()
                .filter(|c: &char| c.is_ascii_hexdigit())
                .repeated()
                .at_least(1)
                .collect::<String>()
                .validate(|s, _e, emit| {
                    u32::from_str_radix(&s, 16)
                        .ok()
                        .and_then(char::from_u32)
                        .unwrap()
                    // .ok_or_else(|| emit(Simple::custom(span, "Invalid hex escape")))
                }),
        ),
        just('o').ignore_then(
            any()
                .filter(|c: &char| c.is_digit(8))
                .repeated()
                .at_least(1)
                .collect::<String>()
                .validate(|s, _e, emit| {
                    u32::from_str_radix(&s, 8)
                        .ok()
                        .and_then(char::from_u32)
                        .unwrap()
                    // .ok_or_else(|| emit(Simple::custom(span, "Invalid octal escape")))
                }),
        ),
        any()
            .filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .at_least(1)
            .collect::<String>()
            .validate(|s, _e, emit| {
                s.parse::<u32>().ok().and_then(char::from_u32).unwrap()
                // .ok_or_else(|| emit(Simple::custom(span, "Invalid decimal escape")))
            }),
    )));

    let regular_char = none_of("\\\"");

    choice((regular_char, escape_sequence))
        .repeated()
        .collect::<String>()
}
