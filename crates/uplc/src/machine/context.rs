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
        &'a [&'a Term<'a>],
        BumpVec<'a, &'a Value<'a>>,
        &'a Context<'a>,
    ),
    FrameCases(&'a Env<'a>, &'a [&'a Term<'a>], &'a Context<'a>),
    NoFrame,
}

impl<'a> Context<'a> {
    pub fn no_frame(arena: &'a Bump) -> &'a Context<'a> {
        arena.alloc(Context::NoFrame)
    }

    pub fn frame_await_arg(
        arena: &'a Bump,
        function: &'a Value<'a>,
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameAwaitArg(function, context))
    }

    pub fn frame_await_fun_term(
        arena: &'a Bump,
        arg_env: &'a Env<'a>,
        argument: &'a Term<'a>,
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameAwaitFunTerm(arg_env, argument, context))
    }

    pub fn frame_await_fun_value(
        arena: &'a Bump,
        argument: &'a Value<'a>,
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameAwaitFunValue(argument, context))
    }

    pub fn frame_force(arena: &'a Bump, context: &'a Context<'a>) -> &'a Context<'a> {
        arena.alloc(Context::FrameForce(context))
    }

    pub fn frame_constr_empty(
        arena: &'a Bump,
        env: &'a Env<'a>,
        index: usize,
        terms: &'a [&'a Term<'a>],
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameConstr(
            env,
            index,
            terms,
            BumpVec::new_in(arena),
            context,
        ))
    }

    pub fn frame_constr(
        arena: &'a Bump,
        env: &'a Env<'a>,
        index: usize,
        terms: &'a [&'a Term<'a>],
        values: BumpVec<'a, &'a Value<'a>>,
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameConstr(env, index, terms, values, context))
    }

    pub fn frame_cases(
        arena: &'a Bump,
        env: &'a Env<'a>,
        terms: &'a [&'a Term<'a>],
        context: &'a Context<'a>,
    ) -> &'a Context<'a> {
        arena.alloc(Context::FrameCases(env, terms, context))
    }
}
