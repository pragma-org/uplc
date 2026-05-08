//! De Bruijn index binder strategy.

use crate::arena::Arena;

use super::{Binder, Eval};

/// A De Bruijn index variable reference.
///
/// The index represents the number of lambda abstractions between the variable occurrence
/// and its binding site (1-based: index 1 refers to the immediately enclosing lambda).
#[derive(Debug, Eq, PartialEq)]
pub struct DeBruijn(usize);

impl DeBruijn {
    /// Allocates a De Bruijn index.
    pub fn new(arena: &Arena, i: usize) -> &Self {
        arena.alloc(DeBruijn(i))
    }

    /// Allocates a De Bruijn index of 0 (used as a placeholder for lambda parameters).
    pub fn zero(arena: &Arena) -> &Self {
        arena.alloc(DeBruijn(0))
    }
}

impl<'a> Binder<'a> for DeBruijn {
    fn var_encode(&self, e: &mut crate::flat::Encoder) -> Result<(), crate::flat::FlatEncodeError> {
        e.word(self.0);

        Ok(())
    }

    fn var_decode(
        arena: &'a Arena,
        d: &mut crate::flat::Decoder,
    ) -> Result<&'a Self, crate::flat::FlatDecodeError> {
        let i = d.word()?;

        let d = DeBruijn::new(arena, i);

        Ok(d)
    }

    fn parameter_encode(
        &self,
        _e: &mut crate::flat::Encoder,
    ) -> Result<(), crate::flat::FlatEncodeError> {
        Ok(())
    }

    fn parameter_decode(
        arena: &'a Arena,
        _d: &mut crate::flat::Decoder,
    ) -> Result<&'a Self, crate::flat::FlatDecodeError> {
        let d = DeBruijn::new(arena, 0);

        Ok(d)
    }
}

impl Eval<'_> for DeBruijn {
    fn index(&self) -> usize {
        self.0
    }
}
