use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatEncodeError {
    #[error("Overflow detected, cannot fit {byte} in {num_bits} bits.")]
    Overflow { byte: u8, num_bits: usize },
}
