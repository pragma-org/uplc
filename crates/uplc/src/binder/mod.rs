//! Variable-binding strategies for UPLC terms.
//!
//! UPLC supports multiple ways to represent variables:
//!
//! - [`DeBruijn`] — the canonical on-chain representation using De Bruijn indices.
//! - [`Name`] — human-readable named bindings used during parsing and pretty-printing.
//! - [`NamedDeBruijn`] — a hybrid that carries both a name and a De Bruijn index.
//!
//! The [`Binder`] trait abstracts over the Flat encoding/decoding of each strategy,
//! while [`Eval`] adds the index lookup required by the CEK machine.

mod debruijn;
mod name;
mod named_debruijn;

pub use debruijn::*;
pub use name::*;
pub use named_debruijn::*;

use crate::{arena::Arena, flat};

/// Abstracts over variable-binding strategies for Flat encoding and decoding.
pub trait Binder<'a>: std::fmt::Debug {
    /// Encodes a variable occurrence (reference site) into the Flat stream.
    fn var_encode(&self, e: &mut flat::Encoder) -> Result<(), flat::FlatEncodeError>;
    /// Decodes a variable occurrence from the Flat stream.
    fn var_decode(
        arena: &'a Arena,
        d: &mut flat::Decoder,
    ) -> Result<&'a Self, flat::FlatDecodeError>;

    /// Encodes a lambda parameter (binding site) into the Flat stream.
    fn parameter_encode(&self, e: &mut flat::Encoder) -> Result<(), flat::FlatEncodeError>;
    /// Decodes a lambda parameter from the Flat stream.
    fn parameter_decode(
        arena: &'a Arena,
        d: &mut flat::Decoder,
    ) -> Result<&'a Self, flat::FlatDecodeError>;
}

/// Extends [`Binder`] with the De Bruijn index lookup required by the CEK machine.
pub trait Eval<'a>: Binder<'a> {
    /// Returns the De Bruijn index (1-based distance to the enclosing lambda).
    fn index(&self) -> usize;
}
