#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DefaultFunction {
    AddInteger,
    EqualsInteger,
    AddByteString,
    EqualsByteString,
}

impl DefaultFunction {
    pub fn force_count(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 0,
            DefaultFunction::EqualsInteger => 0,
            DefaultFunction::AddByteString => 0,
            DefaultFunction::EqualsByteString => 0,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 2,
            DefaultFunction::EqualsInteger => 2,
            DefaultFunction::AddByteString => 2,
            DefaultFunction::EqualsByteString => 2,
        }
    }
}
