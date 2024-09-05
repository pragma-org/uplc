use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::term::Term;

use super::{env::Env, value::Value};

pub enum Context<'a> {
    FrameAwaitArg(&'a Value<'a>, &'a Context<'a>),
    FrameAwaitFunTerm(&'a Env<'a>, &'a Term<'a>, &'a Context<'a>),
    FrameAwaitFunValue(&'a Value<'a>, &'a Context<'a>),
    FrameForce(&'a Context<'a>),
    FrameConstr(
        &'a Env<'a>,
        usize,
        BumpVec<'a, Term<'a>>,
        BumpVec<'a, Value<'a>>,
        &'a Context<'a>,
    ),
    FrameCases(&'a Env<'a>, BumpVec<'a, Term<'a>>, &'a Context<'a>),
    NoFrame,
}

impl<'a> Context<'a> {
    pub fn no_frame(arena: &'a Bump) -> &'a Context<'a> {
        arena.alloc(Context::NoFrame)
    }
}
