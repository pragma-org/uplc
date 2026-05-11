//! Named variable binder strategy.

use crate::arena::Arena;

use super::Binder;

/// A human-readable named variable binding.
///
/// Used during parsing and pretty-printing where variable names are preserved.
/// Each binding carries a `text` label and a `unique` integer to disambiguate
/// shadowed names.
#[derive(Debug)]
pub struct Name<'a> {
    text: &'a str,
    unique: usize,
}

impl<'a> Name<'a> {
    /// Allocates a [`Name`] with the given text label and uniqueness index.
    pub fn new(arena: &'a Arena, text: &'a str, unique: usize) -> &'a Self {
        arena.alloc(Name { text, unique })
    }
}

impl<'a> Binder<'a> for Name<'a> {
    fn var_encode(&self, e: &mut crate::flat::Encoder) -> Result<(), crate::flat::FlatEncodeError> {
        e.utf8(self.text)?;
        e.word(self.unique);

        Ok(())
    }

    fn var_decode(
        arena: &'a Arena,
        d: &mut crate::flat::Decoder,
    ) -> Result<&'a Self, crate::flat::FlatDecodeError> {
        let text = d.utf8(arena)?;
        let index = d.word()?;

        let nd = Name::new(arena, text, index);

        Ok(nd)
    }

    fn parameter_encode(
        &self,
        e: &mut crate::flat::Encoder,
    ) -> Result<(), crate::flat::FlatEncodeError> {
        self.var_encode(e)
    }

    fn parameter_decode(
        arena: &'a Arena,
        d: &mut crate::flat::Decoder,
    ) -> Result<&'a Self, crate::flat::FlatDecodeError> {
        Self::var_decode(arena, d)
    }
}
