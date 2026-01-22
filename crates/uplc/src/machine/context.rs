use bumpalo::collections::Vec as BumpVec;

use crate::{arena::Arena, binder::Eval, term::Term};

use super::{env::Env, value::Value};

pub enum Context<'a, V>
where
    V: Eval<'a>,
{
    FrameAwaitArg(&'a Value<'a, V>, &'a Context<'a, V>),
    FrameAwaitFunTerm(&'a Env<'a, V>, &'a Term<'a, V>, &'a Context<'a, V>),
    FrameAwaitFunValue(&'a Value<'a, V>, &'a Context<'a, V>),
    FrameForce(&'a Context<'a, V>),
    FrameConstr(
        &'a Env<'a, V>,
        usize,
        &'a [&'a Term<'a, V>],
        &'a [&'a Value<'a, V>],
        &'a Context<'a, V>,
    ),
    FrameCases(&'a Env<'a, V>, &'a [&'a Term<'a, V>], &'a Context<'a, V>),
    NoFrame,
}

impl<'a, V> Context<'a, V>
where
    V: Eval<'a>,
{
    pub fn no_frame(arena: &'a Arena) -> &'a Context<'a, V> {
        arena.alloc(Context::NoFrame)
    }

    pub fn frame_await_arg(
        arena: &'a Arena,
        function: &'a Value<'a, V>,
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameAwaitArg(function, context))
    }

    pub fn frame_await_fun_term(
        arena: &'a Arena,
        arg_env: &'a Env<'a, V>,
        argument: &'a Term<'a, V>,
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameAwaitFunTerm(arg_env, argument, context))
    }

    pub fn frame_await_fun_value(
        arena: &'a Arena,
        argument: &'a Value<'a, V>,
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameAwaitFunValue(argument, context))
    }

    pub fn frame_force(arena: &'a Arena, context: &'a Context<'a, V>) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameForce(context))
    }

    pub fn frame_constr_empty(
        arena: &'a Arena,
        env: &'a Env<'a, V>,
        index: usize,
        terms: &'a [&'a Term<'a, V>],
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        let empty = BumpVec::new_in(arena.as_bump());
        let empty = arena.alloc(empty);

        arena.alloc(Context::FrameConstr(env, index, terms, empty, context))
    }

    pub fn frame_constr(
        arena: &'a Arena,
        env: &'a Env<'a, V>,
        index: usize,
        terms: &'a [&'a Term<'a, V>],
        values: &'a [&'a Value<'a, V>],
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameConstr(env, index, terms, values, context))
    }

    pub fn frame_cases(
        arena: &'a Arena,
        env: &'a Env<'a, V>,
        terms: &'a [&'a Term<'a, V>],
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V> {
        arena.alloc(Context::FrameCases(env, terms, context))
    }
}
