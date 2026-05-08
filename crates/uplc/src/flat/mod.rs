//! Flat and CBOR binary encoding/decoding for UPLC programs and data.
//!
//! Flat is the canonical on-chain binary format for UPLC scripts; CBOR wrapping is used
//! in the Cardano transaction witness set. This module re-exports [`Encoder`], [`Decoder`],
//! and their associated error types.

mod builtin;
mod data;
mod decode;
mod encode;
pub mod tag;
mod zigzag;

pub use decode::*;
pub use encode::*;
