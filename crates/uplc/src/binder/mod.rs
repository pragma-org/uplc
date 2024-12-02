use bumpalo::Bump;

mod debruijn;
mod name;
mod named_debruijn;

pub use debruijn::*;
pub use name::*;
pub use named_debruijn::*;

use crate::flat;

pub trait Binder: std::fmt::Debug {
    // this might not need to return a Result
    fn var_encode(&self, e: &mut flat::Encoder) -> Result<(), flat::FlatEncodeError>;
    fn var_decode<'a>(
        arena: &'a Bump,
        d: &mut flat::Decoder,
    ) -> Result<&'a Self, flat::FlatDecodeError>;

    // this might not need to return a Result
    fn parameter_encode(&self, e: &mut flat::Encoder) -> Result<(), flat::FlatEncodeError>;
    fn parameter_decode<'a>(
        arena: &'a Bump,
        d: &mut flat::Decoder,
    ) -> Result<&'a Self, flat::FlatDecodeError>;
}

pub trait Eval: Binder {
    fn index(&self) -> usize;
}
