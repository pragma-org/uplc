use bumpalo::Bump;

pub struct Arena {
    bump: Bump,
}

impl Arena {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub fn from_bump(bump: Bump) -> Self {
        Self { bump }
    }

    pub fn into_inner(self) -> Bump {
        self.bump
    }

    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    pub(crate) fn as_bump(&self) -> &Bump {
        &self.bump
    }

    pub fn reset(&mut self) {
        self.bump.reset();
    }
}
