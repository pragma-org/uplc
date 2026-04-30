use crate::{binder::Eval, term::Term};

use super::{env::Env, value::Value};

/// A single continuation frame. No parent pointer — ordering is maintained
/// by position in the ContextStack.
pub enum Frame<'a, V>
where
    V: Eval<'a>,
{
    AwaitArg(&'a Value<'a, V>),
    AwaitFunTerm(&'a Env<'a, V>, &'a Term<'a, V>),
    AwaitFunValue(&'a Value<'a, V>),
    Force,
    AwaitArgForLambda(&'a Term<'a, V>, &'a Env<'a, V>),
    Constr(
        &'a Env<'a, V>,
        usize,
        &'a [&'a Term<'a, V>],
        &'a [&'a Value<'a, V>],
    ),
    Cases(&'a Env<'a, V>, &'a [&'a Term<'a, V>]),
}

/// Pre-allocated stack of continuation frames.
pub struct ContextStack<'a, V>
where
    V: Eval<'a>,
{
    frames: Vec<Frame<'a, V>>,
}

impl<'a, V> ContextStack<'a, V>
where
    V: Eval<'a>,
{
    pub fn new() -> Self {
        ContextStack {
            frames: Vec::with_capacity(64),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    #[inline]
    pub fn push(&mut self, frame: Frame<'a, V>) {
        self.frames.push(frame);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Frame<'a, V>> {
        self.frames.pop()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }
}
