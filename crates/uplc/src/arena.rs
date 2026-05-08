//! Arena allocator for zero-copy UPLC term construction.
//!
//! [`Arena`] wraps [`bumpalo::Bump`] for general term allocation and stores
//! [`Integer`] values in a stable append-only vector so that
//! raw references into them remain valid across further allocations.

use std::any::type_name;

use append_only_vec::AppendOnlyVec;
use bumpalo::Bump;

use crate::constant::Integer;

/// Arena allocator for zero-copy UPLC term construction.
///
/// General allocations go through the bump allocator; [`Integer`]
/// values are stored separately in a stable append-only vector so raw references into them
/// remain valid across further allocations.
pub struct Arena {
    bump: Bump,
    integers: AppendOnlyVec<Integer>,
}

impl Arena {
    /// Creates a new empty arena.
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            integers: AppendOnlyVec::new(),
        }
    }

    /// Creates an arena reusing an existing [`bumpalo::Bump`] allocator.
    pub fn from_bump(bump: Bump) -> Self {
        Self {
            bump,
            integers: AppendOnlyVec::new(),
        }
    }

    /// Allocates `value` in the arena and returns a mutable reference to it.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `T` is [`Integer`]; use [`Arena::alloc_integer`] instead.
    pub fn alloc<T>(&self, value: T) -> &mut T {
        if cfg!(debug_assertions) {
            assert!(
                type_name::<T>() != type_name::<Integer>(),
                "use alloc_integer for Integer types"
            );
        }
        self.bump.alloc(value)
    }

    /// Allocates an [`Integer`] with a stable address.
    ///
    /// Unlike the bump allocator, integers are stored in an append-only vector
    /// so that existing references remain valid after subsequent allocations.
    pub fn alloc_integer(&self, value: Integer) -> &Integer {
        let idx = self.integers.push(value);
        &self.integers[idx]
    }

    pub(crate) fn as_bump(&self) -> &Bump {
        &self.bump
    }

    /// Resets the arena, freeing all allocated values.
    ///
    /// All references previously returned by this arena are invalidated.
    pub fn reset(&mut self) {
        self.integers = AppendOnlyVec::new();
        self.bump.reset();
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
