use bumpalo::Bump;

use crate::term::Term;

use super::{context::Context, env::Env, value::Value};

pub enum MachineState<'a> {
    Return(&'a Context<'a>, &'a Value<'a>),
    Compute(&'a Context<'a>, &'a Env<'a>, &'a Term<'a>),
    Done(&'a Term<'a>),
}

impl<'a> MachineState<'a> {
    pub fn compute(
        arena: &'a Bump,
        context: &'a Context<'a>,
        env: &'a Env<'a>,
        term: &'a Term<'a>,
    ) -> &'a mut MachineState<'a> {
        arena.alloc(MachineState::Compute(context, env, term))
    }

    pub fn return_(
        arena: &'a Bump,
        context: &'a Context<'a>,
        value: &'a Value<'a>,
    ) -> &'a mut MachineState<'a> {
        arena.alloc(MachineState::Return(context, value))
    }

    pub fn done(arena: &'a Bump, term: &'a Term<'a>) -> &'a mut MachineState<'a> {
        arena.alloc(MachineState::Done(term))
    }
}
