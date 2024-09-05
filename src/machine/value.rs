use bumpalo::Bump;

use crate::{
    constant::{Constant, Integer},
    term::Term,
    typ::Type,
};
use bumpalo::collections::Vec as BumpVec;

use super::{env::Env, runtime::Runtime, MachineError};

#[derive(Debug)]
pub enum Value<'a> {
    Con(&'a Constant<'a>),
    Lambda {
        parameter: usize,
        body: &'a Term<'a>,
        env: &'a Env<'a>,
    },
    Builtin(&'a Runtime<'a>),
    Delay(&'a Term<'a>, &'a Env<'a>),
    Constr(usize, BumpVec<'a, &'a Value<'a>>),
}

impl<'a> Value<'a> {
    pub fn con(arena: &'a Bump, constant: &'a Constant<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Con(constant))
    }

    pub fn lambda(
        arena: &'a Bump,
        parameter: usize,
        body: &'a Term<'a>,
        env: &'a Env<'a>,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Lambda {
            parameter,
            body,
            env,
        })
    }

    pub fn delay(arena: &'a Bump, body: &'a Term<'a>, env: &'a Env<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Delay(body, env))
    }

    pub fn constr_empty(arena: &'a Bump, tag: usize) -> &'a Value<'a> {
        arena.alloc(Value::Constr(tag, BumpVec::new_in(arena)))
    }

    pub fn constr_push(
        arena: &'a Bump,
        tag: usize,
        value: &'a Value<'a>,
        values: &'a BumpVec<'a, &'a Value<'a>>,
    ) -> &'a Value<'a> {
        let mut values = values.clone();

        values.push(value);

        arena.alloc(Value::Constr(tag, values))
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Integer(i));

        Value::con(arena, con)
    }

    pub fn builtin(arena: &'a Bump, runtime: &'a Runtime<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Builtin(runtime))
    }

    pub fn unwrap_integer(&'a self) -> Result<&'a Integer, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Integer(integer) = inner else {
            return Err(MachineError::TypeMismatch(Type::Integer, inner));
        };

        Ok(integer)
    }

    pub fn unwrap_constant(&'a self) -> Result<&'a Constant<'a>, MachineError<'a>> {
        let Value::Con(item) = self else {
            return Err(MachineError::NotAConstant(self));
        };

        Ok(item)
    }
}
