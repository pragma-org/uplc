#[derive(Debug)]
pub enum Type<'a> {
    Bool,
    Integer,
    String,
    ByteString,
    Unit,
    List(&'a Type<'a>),
    Pair(&'a Type<'a>, &'a Type<'a>),
    Data,
    Bls12_381G1Element,
    Bls12_381G2Element,
    Bls12_381MlResult,
}
