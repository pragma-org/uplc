use bumpalo::collections::Vec as BumpVec;

use crate::term::Term;

use super::{env::Env, value::Value};

pub enum Context<'a> {
    FrameAwaitArg(&'a Value<'a>, &'a Context<'a>),
    FrameAwaitFunTerm(Env<'a>, &'a Term<'a>, &'a Context<'a>),
    FrameAwaitFunValue(&'a Value<'a>, &'a Context<'a>),
    FrameForce(&'a Context<'a>),
    FrameConstr(
        Env<'a>,
        usize,
        BumpVec<'a, Term<'a>>,
        BumpVec<'a, Value<'a>>,
        &'a Context<'a>,
    ),
    FrameCases(Env<'a>, BumpVec<'a, Term<'a>>, &'a Context<'a>),
    NoFrame,
}
