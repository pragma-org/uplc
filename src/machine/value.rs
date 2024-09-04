use crate::{constant::Constant, term::Term};

use super::env::Env;

pub enum Value<'a> {
    Con(&'a Constant<'a>),
    Lambda {
        parameter: usize,
        body: &'a Term<'a>,
        env: &'a Env<'a>,
    },
}
