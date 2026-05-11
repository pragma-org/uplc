#![warn(missing_docs)]
//! A lightning-fast [UPLC] (Untyped Plutus Language Core) evaluator implemented as a CEK machine.
//!
//! UPLC is the low-level bytecode compiled from [Plutus], the smart-contract language for the
//! [Cardano] blockchain. This crate provides a complete evaluation pipeline: parsing,
//! binary encoding/decoding (Flat/CBOR), and execution against configurable cost models for
//! Plutus V1, V2, and V3.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use amaru_uplc::{
//!     arena::Arena,
//!     binder::DeBruijn,
//!     program::{Program, Version},
//!     term::Term,
//! };
//!
//! let arena = Arena::new();
//!
//! // Build a term: addInteger 1 3
//! let term = Term::add_integer(&arena)
//!     .apply(&arena, Term::integer_from(&arena, 1))
//!     .apply(&arena, Term::integer_from(&arena, 3));
//!
//! let version = Version::plutus_v3(&arena);
//! let program = Program::<DeBruijn>::new(&arena, version, term);
//! let result = program.eval(&arena);
//!
//! assert_eq!(result.term.unwrap(), Term::integer_from(&arena, 4));
//! ```
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`arena`] | Arena allocator wrapping [`bumpalo`] with stable integer storage |
//! | [`program`] | Top-level [`Program`](program::Program) and UPLC [`Version`](program::Version) |
//! | [`term`] | [`Term`](term::Term) AST with builder helpers |
//! | [`binder`] | Variable-binding strategies: De Bruijn indices, named, and named-De Bruijn |
//! | [`constant`] | Compile-time constant values |
//! | [`data`] | Plutus structured data ([`PlutusData`](data::PlutusData)) |
//! | [`builtin`] | All built-in functions ([`DefaultFunction`](builtin::DefaultFunction)) |
//! | [`machine`] | CEK machine, cost models, and evaluation results |
//! | [`syn`] | UPLC text-format parser |
//! | [`flat`] | Flat/CBOR binary encoder and decoder |
//! | [`bls`] | BLS12-381 elliptic curve primitives |
//! | [`typ`] | UPLC type system |
//!
//! [UPLC]: https://plutus.readthedocs.io/en/latest/reference/uplc-introduction.html
//! [Plutus]: https://plutus.readthedocs.io/
//! [Cardano]: https://cardano.org/

pub mod arena;
pub mod binder;
pub mod bls;
pub mod builtin;
pub mod constant;
pub mod data;
pub mod flat;
pub mod machine;
pub mod program;
pub mod syn;
pub mod term;
pub mod typ;

pub use bumpalo;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::binder::DeBruijn;
    use crate::program::Version;

    use super::arena::Arena;
    use super::program::Program;
    use super::term::Term;

    #[test]
    fn add_integer() {
        let arena = Arena::new();

        let term = Term::add_integer(&arena)
            .apply(&arena, Term::integer_from(&arena, 1))
            .apply(&arena, Term::integer_from(&arena, 3));

        let version = Version::plutus_v3(&arena);

        let program = Program::<DeBruijn>::new(&arena, version, term);

        let result = program.eval(&arena);

        assert_eq!(result.term.unwrap(), Term::integer_from(&arena, 4));
    }

    #[test]
    fn fibonacci() {
        let arena = &Arena::new();

        let double_force = Term::var(arena, DeBruijn::new(arena, 1))
            .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
            .lambda(arena, DeBruijn::zero(arena))
            .delay(arena)
            .force(arena)
            .apply(
                arena,
                Term::var(arena, DeBruijn::new(arena, 3))
                    .apply(
                        arena,
                        Term::var(arena, DeBruijn::new(arena, 1))
                            .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
                            .lambda(arena, DeBruijn::zero(arena))
                            .delay(arena)
                            .force(arena)
                            .apply(arena, Term::var(arena, DeBruijn::new(arena, 2))),
                    )
                    .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
                    .lambda(arena, DeBruijn::zero(arena))
                    .lambda(arena, DeBruijn::zero(arena)),
            )
            .lambda(arena, DeBruijn::zero(arena))
            .delay(arena)
            .delay(arena)
            .force(arena)
            .force(arena);

        let if_condition = Term::if_then_else(arena)
            .force(arena)
            .apply(arena, Term::var(arena, DeBruijn::new(arena, 3)))
            .apply(arena, Term::var(arena, DeBruijn::new(arena, 2)))
            .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
            .apply(arena, Term::unit(arena))
            .lambda(arena, DeBruijn::zero(arena))
            .lambda(arena, DeBruijn::zero(arena))
            .lambda(arena, DeBruijn::zero(arena))
            .delay(arena)
            .force(arena);

        let add = Term::add_integer(arena)
            .apply(
                arena,
                Term::var(arena, DeBruijn::new(arena, 3)).apply(
                    arena,
                    Term::subtract_integer(arena)
                        .apply(arena, Term::var(arena, DeBruijn::new(arena, 2)))
                        .apply(arena, Term::integer_from(arena, 1)),
                ),
            )
            .apply(
                arena,
                Term::var(arena, DeBruijn::new(arena, 3)).apply(
                    arena,
                    Term::subtract_integer(arena)
                        .apply(arena, Term::var(arena, DeBruijn::new(arena, 2)))
                        .apply(arena, Term::integer_from(arena, 2)),
                ),
            )
            .lambda(arena, DeBruijn::zero(arena));

        let term = double_force
            .apply(
                arena,
                if_condition
                    .apply(
                        arena,
                        Term::less_than_equals_integer(arena)
                            .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
                            .apply(arena, Term::integer_from(arena, 1)),
                    )
                    .apply(
                        arena,
                        Term::var(arena, DeBruijn::new(arena, 2))
                            .lambda(arena, DeBruijn::zero(arena)),
                    )
                    .apply(arena, add)
                    .lambda(arena, DeBruijn::zero(arena))
                    .lambda(arena, DeBruijn::zero(arena)),
            )
            .apply(arena, Term::var(arena, DeBruijn::new(arena, 1)))
            .lambda(arena, DeBruijn::zero(arena))
            .apply(arena, Term::integer_from(arena, 15));

        let version = Version::plutus_v3(arena);

        let program = Program::new(arena, version, term);

        let result = program.eval(arena);

        assert_eq!(result.term.unwrap(), Term::integer_from(arena, 610));
    }
}
