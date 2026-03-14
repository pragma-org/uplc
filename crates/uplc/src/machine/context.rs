use bumpalo::collections::Vec as BumpVec;

use crate::{arena::Arena, binder::Eval, term::Term};

use super::{env::Env, value::Value};

/// A single continuation frame. No parent pointer — ordering is maintained
/// by position in the ContextStack.
pub enum Frame<'a, V>
where
    V: Eval<'a>,
{
    FrameAwaitArg(&'a Value<'a, V>),
    FrameAwaitFunTerm(&'a Env<'a, V>, &'a Term<'a, V>),
    FrameAwaitFunValue(&'a Value<'a, V>),
    FrameForce,
    FrameAwaitArgForLambda(&'a Term<'a, V>, &'a Env<'a, V>),
    FrameConstr(
        &'a Env<'a, V>,
        usize,
        &'a [&'a Term<'a, V>],
        &'a [&'a Value<'a, V>],
    ),
    FrameCases(&'a Env<'a, V>, &'a [&'a Term<'a, V>]),
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
