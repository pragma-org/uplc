use chumsky::{prelude::*, Parser};

use crate::program::Version;

use super::types::{Context, Extra, MapExtra};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a mut Version<'a>, Extra<'a>> {
    text::int(10)
        .map(|v: &str| v.parse().unwrap())
        .then(text::int(10).map(|v: &str| v.parse().unwrap()))
        .then(text::int(10).map(|v: &str| v.parse().unwrap()))
        .validate(|((major, minor), patch), e: &mut MapExtra<'a, '_>, emit| {
            let state = e.state();

            let version = Version::new(state.arena, major, minor, patch);

            if version.is_v1_0_0() {
                state.set_context(Context::V1_0_0)
            } else if version.is_v1_1_0() {
                state.set_context(Context::V1_1_0)
            } else {
                // TODO: emit invalid version erro
                // emit(Simple);
            }

            version
        })
}
