use chumsky::prelude::*;

use crate::typ::Type;

use super::types::{Extra, MapExtra};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a Type<'a>, Extra<'a>> {
    recursive(|rec_typ| {
        choice((
            // integer
            text::keyword("integer")
                .ignored()
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::integer(state.arena)
                }),
            // bool
            text::keyword("bool")
                .ignored()
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::bool(state.arena)
                }),
            // bytestring
            text::keyword("bytestring")
                .ignored()
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::byte_string(state.arena)
                }),
            // string
            text::keyword("string")
                .ignored()
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::string(state.arena)
                }),
            // list
            text::keyword("list")
                .padded()
                .ignore_then(rec_typ.padded())
                .delimited_by(just('('), just(')'))
                .map_with(|typ, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::list(state.arena, typ)
                }),
            // data
            text::keyword("data")
                .ignored()
                .map_with(|_, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    Type::data(state.arena)
                }),
        ))
        .boxed()
    })
}
