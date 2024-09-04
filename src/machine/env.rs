use bumpalo::{collections::Vec as BumpVec, Bump};

use super::value::Value;

#[derive(Debug)]
pub struct Env<'a>(BumpVec<'a, Value<'a>>);

impl<'a> Env<'a> {
    pub fn new_in(arena: &'a Bump) -> &'a Self {
        arena.alloc(Self(BumpVec::new_in(arena)))
    }

    pub fn lookup(&'a self, name: usize) -> Option<&'a Value<'a>> {
        self.0.get(self.0.len() - name)
    }
}
