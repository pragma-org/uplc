//! Named De Bruijn variable binder strategy.

use crate::arena::Arena;

use super::{Binder, Eval};

/// A hybrid variable binding that carries both a human-readable name and a De Bruijn index.
///
/// Useful when debugging or round-tripping through a textual format: the `text` label
/// aids readability while the `index` drives CEK machine variable lookup.
#[derive(Debug)]
pub struct NamedDeBruijn<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> NamedDeBruijn<'a> {
    /// Allocates a [`NamedDeBruijn`] with the given text label and De Bruijn index.
    pub fn new(arena: &'a Arena, text: &'a str, index: usize) -> &'a Self {
        arena.alloc(NamedDeBruijn { text, index })
    }
}

impl<'a> Binder<'a> for NamedDeBruijn<'a> {
    fn var_encode(&self, e: &mut crate::flat::Encoder) -> Result<(), crate::flat::FlatEncodeError> {
        e.utf8(self.text)?;
        e.word(self.index);

        Ok(())
    }

    fn var_decode(
        arena: &'a Arena,
        d: &mut crate::flat::Decoder,
    ) -> Result<&'a Self, crate::flat::FlatDecodeError> {
        let text = d.utf8(arena)?;
        let index = d.word()?;

        let nd = NamedDeBruijn::new(arena, text, index);

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

impl<'a> Eval<'a> for NamedDeBruijn<'a> {
    fn index(&self) -> usize {
        self.index
    }
}
