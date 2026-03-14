use crate::{arena::Arena, binder::Eval};

use super::value::Value;

/// Spine size — number of values stored inline per Env node.
/// Covers the vast majority of lookups (de Bruijn index 1-8)
/// in a single cache-friendly array access.
const SPINE_SIZE: usize = 8;

#[derive(Debug)]
pub struct Env<'a, V>
where
    V: Eval<'a>,
{
    /// Values stored in this spine, most recent first.
    /// Only `len` entries are valid.
    values: [Option<&'a Value<'a, V>>; SPINE_SIZE],
    len: u8,
    /// Parent env for lookups beyond this spine.
    parent: Option<&'a Env<'a, V>>,
}

impl<'a, V> Env<'a, V>
where
    V: Eval<'a>,
{
    pub fn new_in(arena: &'a Arena) -> &'a Self {
        arena.alloc(Self {
            values: [None; SPINE_SIZE],
            len: 0,
            parent: None,
        })
    }

    pub fn push(&'a self, arena: &'a Arena, arg: &'a Value<'a, V>) -> &'a Self {
        if (self.len as usize) < SPINE_SIZE {
            // Room in current spine — copy and append
            let mut new_values = self.values;
            new_values[self.len as usize] = Some(arg);
            arena.alloc(Self {
                values: new_values,
                len: self.len + 1,
                parent: self.parent,
            })
        } else {
            // Spine full — start a new spine with this env as parent
            let mut new_values = [None; SPINE_SIZE];
            new_values[0] = Some(arg);
            arena.alloc(Self {
                values: new_values,
                len: 1,
                parent: Some(self),
            })
        }
    }

    // De Bruijn indices are 1-based
    #[inline]
    pub fn lookup(&self, index: usize) -> Option<&'a Value<'a, V>> {
        if index == 0 {
            return None;
        }

        let idx = index - 1; // Convert to 0-based
        let len = self.len as usize;

        if idx < len {
            // Fast path: value is in this spine (most common case)
            // Values are stored newest-first, so index 0 (de Bruijn 1) is at position len-1
            self.values[len - 1 - idx]
        } else if let Some(parent) = self.parent {
            // Recurse into parent spine
            parent.lookup(index - len)
        } else {
            None
        }
    }
}
