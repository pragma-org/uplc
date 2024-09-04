use bumpalo::{collections::Vec as BumpVec, Bump};

use super::{value::Value, MachineError};
use crate::term::Term;

pub struct Env<'a>(BumpVec<'a, Value<'a>>);

impl<'a> Env<'a> {
    pub fn new_in(arena: &'a Bump) -> Self {
        Self(BumpVec::new_in(arena))
    }

    pub fn lookup(&'a self, term: &'a Term<'a>) -> Result<&'a Value<'a>, MachineError<'a>> {
        let Term::Var(name) = term else {
            unreachable!("this should never happen");
        };

        self.0
            .get(self.0.len() - name)
            .ok_or_else(|| MachineError::OpenTermEvaluated(term))
    }
}
