use std::any::type_name;

use append_only_vec::AppendOnlyVec;
use bumpalo::Bump;

use crate::constant::Integer;

pub struct Arena {
    bump: Bump,
    integers: AppendOnlyVec<Integer>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            integers: AppendOnlyVec::new(),
        }
    }

    pub fn from_bump(bump: Bump) -> Self {
        Self {
            bump,
            integers: AppendOnlyVec::new(),
        }
    }

    pub fn alloc<T>(&self, value: T) -> &mut T {
        if cfg!(debug_assertions) {
            assert!(
                type_name::<T>() != type_name::<Integer>(),
                "use alloc_integer for Integer types"
            );
        }
        self.bump.alloc(value)
    }

    pub fn alloc_integer(&self, value: Integer) -> &Integer {
        let idx = self.integers.push(value);
        &self.integers[idx]
    }

    pub(crate) fn as_bump(&self) -> &Bump {
        &self.bump
    }

    pub fn reset(&mut self) {
        // Drop all allocated integers
        self.integers = AppendOnlyVec::new();
        self.bump.reset();
    }
}
