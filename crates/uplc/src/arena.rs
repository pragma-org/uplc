use std::{any::type_name, cell::RefCell};

use bumpalo::{boxed::Box as BumpBox, Bump};

use crate::constant::Integer;

pub struct Arena {
    bump: Bump,
    integers: RefCell<Vec<*mut Integer>>,
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            integers: RefCell::new(Vec::new()),
        }
    }

    pub fn from_bump(bump: Bump) -> Self {
        Self {
            bump,
            integers: RefCell::new(Vec::new()),
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
        let boxed = BumpBox::new_in(value, &self.bump);
        let ptr = BumpBox::leak(boxed);
        self.integers.borrow_mut().push(ptr);
        ptr
    }

    pub(crate) fn as_bump(&self) -> &Bump {
        &self.bump
    }

    pub fn reset(&mut self) {
        while let Some(ptr) = self.integers.borrow_mut().pop() {
            unsafe {
                drop(BumpBox::from_raw(ptr));
            }
        }
        self.bump.reset();
    }
}
