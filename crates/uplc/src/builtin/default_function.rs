#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DefaultFunction {
    AddInteger,
    SubtractInteger,
    EqualsInteger,
    LessThanEqualsInteger,
    AddByteString,
    EqualsByteString,
    IfThenElse,
}

impl DefaultFunction {
    pub fn force_count(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 0,
            DefaultFunction::SubtractInteger => 0,
            DefaultFunction::EqualsInteger => 0,
            DefaultFunction::LessThanEqualsInteger => 0,
            DefaultFunction::AddByteString => 0,
            DefaultFunction::EqualsByteString => 0,
            DefaultFunction::IfThenElse => 1,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 2,
            DefaultFunction::SubtractInteger => 2,
            DefaultFunction::EqualsInteger => 2,
            DefaultFunction::LessThanEqualsInteger => 2,
            DefaultFunction::AddByteString => 2,
            DefaultFunction::EqualsByteString => 2,
            DefaultFunction::IfThenElse => 3,
        }
    }
}
