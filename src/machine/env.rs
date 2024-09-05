use bumpalo::{collections::Vec as BumpVec, Bump};

use super::value::Value;

#[derive(Debug)]
pub struct Env<'a>(BumpVec<'a, &'a Value<'a>>);

impl<'a> Env<'a> {
    pub fn new_in(arena: &'a Bump) -> &'a Self {
        arena.alloc(Self(BumpVec::new_in(arena)))
    }

    pub fn push(&'a self, arena: &'a Bump, argument: &'a Value<'a>) -> &'a mut Self {
        let mut new_env = self.0.clone();

        new_env.push(argument);

        arena.alloc(Self(new_env))
    }

    pub fn lookup(&'a self, name: usize) -> Option<&&'a Value<'a>> {
        self.0.get(self.0.len() - name)
    }
}
