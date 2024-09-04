use crate::constant::Constant;

pub enum Value<'a> {
    Con(&'a Constant<'a>),
}
