use bumpalo::{collections::Vec as BumpVec, Bump};
use rug::Assign;

use crate::{
    builtin::DefaultFunction,
    constant::{self, Integer},
};

use super::{value::Value, MachineError};

#[derive(Debug)]
pub struct Runtime<'a> {
    pub args: BumpVec<'a, &'a Value<'a>>,
    pub fun: &'a DefaultFunction,
    pub forces: usize,
}

impl<'a> Runtime<'a> {
    pub fn needs_force(&self) -> bool {
        self.forces < self.fun.force_count()
    }

    pub fn is_arrow(&self) -> bool {
        self.args.len() < self.fun.arity()
    }

    pub fn is_ready(&self) -> bool {
        self.args.len() == self.fun.arity()
    }

    pub fn call(&self, arena: &'a Bump) -> Result<&'a Value<'a>, MachineError<'a>> {
        match self.fun {
            DefaultFunction::AddInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 + arg2;

                let new = constant::integer(arena);

                new.assign(result);

                let value = Value::integer(arena, new);

                Ok(value)
            }
            DefaultFunction::EqualsInteger => todo!(),
            DefaultFunction::AddByteString => todo!(),
            DefaultFunction::EqualsByteString => todo!(),
        }
    }
}
