use bumpalo::Bump;

use crate::{constant::Constant, term::Term, typ::Type};

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
}

impl<'a> Value<'a> {
    pub fn unwrap_integer(&'a self) -> Result<i128, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Integer(integer) = inner else {
            return Err(MachineError::TypeMismatch(Type::Integer, inner));
        };

        Ok(*integer)
    }

    pub fn integer(arena: &'a Bump, i: i128) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Integer(i));
        arena.alloc(Value::Con(con))
    }

    pub fn unwrap_constant(&'a self) -> Result<&'a Constant<'a>, MachineError<'a>> {
        let Value::Con(item) = self else {
            return Err(MachineError::NotAConstant(self));
        };

        Ok(item)
    }
}
