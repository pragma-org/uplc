use bumpalo::collections::Vec as BumpVec;
use chumsky::prelude::*;

use crate::data::PlutusData;

use super::{
    types::{Extra, MapExtra},
    utils::hex_bytes,
};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a PlutusData<'a>, Extra<'a>> {
    recursive(|data| {
        choice((
            just('B')
                .padded()
                .ignore_then(just('#').ignore_then(hex_bytes()).padded())
                .map_with(|v, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    let bytes = BumpVec::from_iter_in(v, state.arena);

                    PlutusData::byte_string(state.arena, bytes)
                }),
            just('I')
                .padded()
                .ignore_then(text::int(10).padded())
                .map_with(|v, e: &mut MapExtra<'a, '_>| {
                    let state = e.state();

                    PlutusData::integer_from(state.arena, v.parse().unwrap())
                }),
            just("Constr")
                .padded()
                .ignore_then(text::int(10).padded())
                .then(
                    data.separated_by(just(',').padded())
                        .collect()
                        .delimited_by(just('['), just(']')),
                )
                .map_with(
                    |(tag, fields): (_, Vec<&PlutusData<'_>>), e: &mut MapExtra<'a, '_>| {
                        let state = e.state();

                        let fields = BumpVec::from_iter_in(fields, state.arena);

                        PlutusData::constr(state.arena, tag.parse().unwrap(), fields)
                    },
                ),
        ))
        .boxed()
    })
}
