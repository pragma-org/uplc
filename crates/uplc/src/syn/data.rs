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
                    dbg!("b", &v);

                    let state = e.state();

                    let bytes = BumpVec::from_iter_in(v, state.arena);

                    PlutusData::byte_string(state.arena, bytes)
                }),
            just('I')
                .padded()
                .ignore_then(
                    text::int(10)
                        .or(just('-').padded().ignore_then(text::int(10)))
                        .padded(),
                )
                .map_with(|v, e: &mut MapExtra<'a, '_>| {
                    dbg!("i", &v);

                    let state = e.state();

                    let value = if let Some(v) = v.strip_prefix('-') {
                        -(v.parse::<i128>().unwrap())
                    } else {
                        v.parse::<i128>().unwrap()
                    };

                    PlutusData::integer_from(state.arena, value)
                }),
            just("Constr")
                .padded()
                .ignore_then(text::int(10).padded())
                .then(
                    data.clone()
                        .separated_by(just(',').padded())
                        .collect()
                        .delimited_by(just('['), just(']')),
                )
                .map_with(
                    |(tag, fields): (_, Vec<&PlutusData<'_>>), e: &mut MapExtra<'a, '_>| {
                        dbg!("c", &tag, &fields);

                        let state = e.state();

                        let fields = BumpVec::from_iter_in(fields, state.arena);

                        PlutusData::constr(state.arena, tag.parse().unwrap(), fields)
                    },
                ),
            just("List")
                .padded()
                .ignore_then(
                    data.clone()
                        .separated_by(just(',').padded())
                        .collect()
                        .delimited_by(just('['), just(']')),
                )
                .map_with(|items: Vec<&PlutusData<'_>>, e: &mut MapExtra<'a, '_>| {
                    dbg!("c", &items);

                    let state = e.state();

                    let fields = BumpVec::from_iter_in(items, state.arena);

                    PlutusData::list(state.arena, fields)
                }),
            just("Map")
                .padded()
                .ignore_then(
                    data.clone()
                        .padded()
                        .then_ignore(just(',').padded())
                        .then(data.padded())
                        .delimited_by(just('('), just(')'))
                        .separated_by(just(',').padded())
                        .collect()
                        .padded()
                        .delimited_by(just('['), just(']'))
                        .padded(),
                )
                .map_with(
                    |items: Vec<(&PlutusData<'_>, &PlutusData<'_>)>, e: &mut MapExtra<'a, '_>| {
                        let state = e.state();

                        let fields = BumpVec::from_iter_in(items, state.arena);

                        PlutusData::map(state.arena, fields)
                    },
                ),
        ))
        .boxed()
    })
}
