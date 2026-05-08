//! UPLC built-in functions.
//!
//! [`DefaultFunction`] enumerates every built-in function available across Plutus V1/V2/V3:
//! arithmetic, byte-string operations, cryptographic hashing and signature verification,
//! string operations, list and pair manipulation, Plutus data constructors and destructors,
//! BLS12-381 curve operations, and bitwise primitives.

mod default_function;

pub use default_function::*;
