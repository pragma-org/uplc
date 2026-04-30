use crate::{binder::Eval, term::Term};

use super::{env::Env, value::Value};

pub enum MachineState<'a, V>
where
    V: Eval<'a>,
{
    Return(&'a Value<'a, V>),
    Compute(&'a Env<'a, V>, &'a Term<'a, V>),
    Done(&'a Term<'a, V>),
}
