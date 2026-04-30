use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatDecodeError {
    #[error("Reached end of buffer")]
    EndOfBuffer,
    #[error("Buffer is not byte aligned")]
    BufferNotByteAligned,
    #[error("Incorrect value of num_bits, must be less than 9")]
    IncorrectNumBits,
    #[error("Not enough data available, required {0} bytes")]
    NotEnoughBytes(usize),
    #[error("Not enough data available, required {0} bits")]
    NotEnoughBits(usize),
    #[error(transparent)]
    DecodeUtf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    DecodeCbor(#[from] minicbor::decode::Error),
    #[error("Decoding u32 to char {0}")]
    DecodeChar(u32),
    #[error("{0}")]
    Message(String),
    #[error("Default Function not found: {0}")]
    DefaultFunctionNotFound(u8),
    #[error("Unknown term constructor tag: {0}")]
    UnknownTermConstructor(u8),
    #[error("Unknown constant constructor tag: {0:#?}")]
    UnknownConstantConstructor(Vec<u8>),
    #[error("Unknown type tags: {0:#?}")]
    UnknownTypeTags(Vec<u8>),
    #[error("Missing type tag")]
    MissingTypeTag,
    #[error("BLS type not supported")]
    BlsTypeNotSupported,
    #[error("Trailing bytes after script: {0} bytes remaining")]
    TrailingBytes(usize),
    #[error("Builtin function {1} (tag {0}) is not available in the given language version")]
    BuiltinNotAvailable(u8, String),
    #[error("Term constructor {1} (tag {0}) is not available before UPLC version 1.1.0")]
    TermNotAvailable(u8, &'static str),
    #[error("Constant type {1} (tag {0}) is not available before UPLC version 1.1.0")]
    ConstantTypeNotAvailable(u8, &'static str),
}
