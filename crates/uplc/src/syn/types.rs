use bumpalo::Bump;
use chumsky::{input, prelude::*};

pub struct State<'a> {
    pub arena: &'a Bump,
    pub env: Vec<&'a str>,
    pub context: Context,
}

impl<'a> State<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            env: Vec::new(),
            context: Context::default(),
        }
    }

    pub fn set_context(&mut self, context: Context) {
        self.context = context;
    }
}

#[derive(Default)]
pub enum Context {
    #[default]
    V1_0_0,
    V1_1_0,
}

pub type Extra<'a> = extra::Full<Rich<'a, char>, State<'a>, Context>;
pub type MapExtra<'a, 'b> = input::MapExtra<'a, 'b, &'a str, Extra<'a>>;
