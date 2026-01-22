use crate::{arena::Arena, binder::Eval};

use super::value::Value;

#[derive(Debug)]
pub enum Env<'a, V>
where
    V: Eval<'a>,
{
    Empty,
    Cons {
        data: &'a Value<'a, V>,
        next: &'a Env<'a, V>,
    },
}

impl<'a, V> Env<'a, V>
where
    V: Eval<'a>,
{
    pub fn new_in(arena: &'a Arena) -> &'a Self {
        arena.alloc(Self::Empty)
    }

    pub fn push(&'a self, arena: &'a Arena, arg: &'a Value<'a, V>) -> &'a Self {
        arena.alloc(Self::Cons {
            data: arg,
            next: self,
        })
    }

    // De Bruijn indices are 1-based
    // So the data at the env[i] is at De Bruijn index i-1
    pub fn lookup(&self, index: usize) -> Option<&'a Value<'a, V>> {
        if index == 0 {
            return None;
        }

        match self {
            Env::Empty => None,
            Env::Cons { data, next: parent } => {
                if index == 1 {
                    return Some(data);
                }

                parent.lookup(index - 1)
            }
        }
    }
}
