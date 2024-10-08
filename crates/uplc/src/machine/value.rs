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

    pub fn constr(
        arena: &'a Bump,
        tag: usize,
        values: BumpVec<'a, &'a Value<'a>>,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Constr(tag, values))
    }

    pub fn builtin(arena: &'a Bump, runtime: &'a Runtime<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Builtin(runtime))
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Integer(i));

        Value::con(arena, con)
    }

    pub fn byte_string(arena: &'a Bump, b: BumpVec<'a, u8>) -> &'a Value<'a> {
        let con = arena.alloc(Constant::ByteString(b));

        Value::con(arena, con)
    }

    pub fn bool(arena: &'a Bump, b: bool) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Boolean(b));

        Value::con(arena, con)
    }

    pub fn unwrap_integer(&'a self) -> Result<&'a Integer, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Integer(integer) = inner else {
            return Err(MachineError::TypeMismatch(Type::Integer, inner));
        };

        Ok(integer)
    }

    pub fn unwrap_byte_string(&'a self) -> Result<&BumpVec<'a, u8>, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::ByteString(byte_string) = inner else {
            return Err(MachineError::TypeMismatch(Type::ByteString, inner));
        };

        Ok(byte_string)
    }

    pub fn unwrap_bool(&'a self) -> Result<bool, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Boolean(b) = inner else {
            return Err(MachineError::TypeMismatch(Type::Bool, inner));
        };

        Ok(*b)
    }

    pub(super) fn unwrap_pair(
        &'a self,
    ) -> Result<
        (
            &'a Type<'a>,
            &'a Type<'a>,
            &'a Constant<'a>,
            &'a Constant<'a>,
        ),
        MachineError<'a>,
    > {
        let inner = self.unwrap_constant()?;

        let Constant::ProtoPair(t1, t2, first, second) = inner else {
            return Err(MachineError::ExpectedPair(inner));
        };

        Ok((t1, t2, first, second))
    }

    pub fn unwrap_constant(&'a self) -> Result<&'a Constant<'a>, MachineError<'a>> {
        let Value::Con(item) = self else {
            return Err(MachineError::NotAConstant(self));
        };

        Ok(item)
    }
}
