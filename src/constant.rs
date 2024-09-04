use bumpalo::collections::Vec as BumpVec;

#[derive(Debug, PartialEq)]
pub enum Constant<'a> {
    Integer(i128),
    ByteString(BumpVec<'a, u8>),
    Boolean(bool),
}
