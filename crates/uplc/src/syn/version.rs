use chumsky::{prelude::*, Parser};

use crate::program::Version;

use super::types::{Extra, MapExtra};

pub fn parser<'a>() -> impl Parser<'a, &'a str, &'a mut Version<'a>, Extra<'a>> {
    text::int(10)
        .map(|v: &str| v.parse().unwrap())
        .then(text::int(10).map(|v: &str| v.parse().unwrap()))
        .then(text::int(10).map(|v: &str| v.parse().unwrap()))
        .validate(|((major, minor), patch), e: &mut MapExtra<'a, '_>, emit| {
            let state = e.state();

            let version = Version::new(state.arena, major, minor, patch);

            if !version.is_valid_version() {
                // emit(Simple);
            }

            version
        })
}
