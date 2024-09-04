use crate::term::Term;

use super::{context::Context, env::Env, value::Value};

pub enum MachineState<'a> {
    Return(&'a Context<'a>, &'a Value<'a>),
    Compute(&'a Context<'a>, Env<'a>, &'a Term<'a>),
    Done(&'a Term<'a>),
}
