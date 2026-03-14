use crate::{binder::Eval, term::Term};

use super::{context::Context, env::Env, value::Value};

pub enum MachineState<'a, V>
where
    V: Eval<'a>,
{
    Return(&'a Context<'a, V>, &'a Value<'a, V>),
    Compute(&'a Context<'a, V>, &'a Env<'a, V>, &'a Term<'a, V>),
    Done(&'a Term<'a, V>),
}
