use bumpalo::{collections::Vec as BumpVec, Bump};
use rug::Assign;

use crate::{
    builtin::DefaultFunction,
    constant::{self},
};

use super::{value::Value, MachineError};

#[derive(Debug)]
pub struct Runtime<'a> {
    pub args: BumpVec<'a, &'a Value<'a>>,
    pub fun: &'a DefaultFunction,
    pub forces: usize,
}

impl<'a> Runtime<'a> {
    pub fn new(arena: &'a Bump, fun: &'a DefaultFunction) -> &'a Self {
        arena.alloc(Self {
            args: BumpVec::new_in(arena),
            fun,
            forces: 0,
        })
    }

    pub fn push(&self, arena: &'a Bump, arg: &'a Value<'a>) -> &'a Self {
        let new_runtime = arena.alloc(Runtime {
            args: self.args.clone(),
            fun: self.fun,
            forces: self.forces,
        });

        new_runtime.args.push(arg);

        new_runtime
    }

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
