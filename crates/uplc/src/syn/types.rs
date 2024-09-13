use bumpalo::Bump;
use chumsky::{input, prelude::*};

pub struct State<'a> {
    pub arena: &'a Bump,
    pub env: Vec<&'a str>,
}

impl<'a> State<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            env: Vec::new(),
        }
    }
}

#[derive(Default)]
pub enum Context {
    V1_0_0,
    #[default]
    V1_1_0,
}

pub type Extra<'a> = extra::Full<Simple<'a, char>, State<'a>, Context>;
pub type MapExtra<'a, 'b> = input::MapExtra<'a, 'b, &'a str, Extra<'a>>;
