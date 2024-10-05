mod data;
pub mod decode;
mod zigzag;

use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};
use decode::{Ctx, Decoder, FlatDecodeError};

use crate::{
    builtin::DefaultFunction,
    constant::Constant,
    program::{Program, Version},
    term::Term,
};

const TERM_TAG_WIDTH: usize = 4;
const CONST_TAG_WIDTH: usize = 4;
const BUILTIN_TAG_WIDTH: usize = 7;

pub fn decode<'a>(arena: &'a Bump, bytes: &[u8]) -> Result<&'a Program<'a>, FlatDecodeError> {
    let mut decoder = Decoder::new(bytes);

    let major = decoder.word()?;
    let minor = decoder.word()?;
    let patch = decoder.word()?;

    let version = Version::new(arena, major, minor, patch);

    let mut ctx = Ctx { arena };

    let term = decode_term(&mut ctx, &mut decoder)?;

    decoder.filler()?;

    // TODO: probably should add a `finish()?` method that errors if bytes remain

    Ok(Program::new(arena, version, term))
}

fn decode_term<'a>(
    ctx: &mut Ctx<'a>,
    decoder: &mut Decoder<'_>,
) -> Result<&'a Term<'a>, FlatDecodeError> {
    let tag = decoder.bits8(TERM_TAG_WIDTH)?;

    match tag {
        // Var
        0 => Ok(Term::var(ctx.arena, decoder.word()?)),
        // Delay
        1 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.delay(ctx.arena))
        }
        // Lambda
        2 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.lambda(ctx.arena, 0))
        }
        // Apply
        3 => {
            let function = decode_term(ctx, decoder)?;
            let argument = decode_term(ctx, decoder)?;

            let term = function.apply(ctx.arena, argument);

            Ok(term)
        }
        // Constant
        4 => {
            let constant = decode_constant(ctx, decoder)?;

            Ok(Term::constant(ctx.arena, constant))
        }
        // Force
        5 => {
            let term = decode_term(ctx, decoder)?;

            Ok(term.force(ctx.arena))
        }
        // Error
        6 => Ok(Term::error(ctx.arena)),
        // Builtin
        7 => {
            let builtin_tag = decoder.bits8(BUILTIN_TAG_WIDTH)?;

            let function = try_from_builtin_tag(ctx.arena, builtin_tag)?;

            let term = Term::builtin(ctx.arena, function);

            Ok(term)
        }
        // Constr
        8 => {
            let tag = decoder.word()?;
            let fields = decoder.list_with(ctx, decode_term)?;

            let term = Term::constr(ctx.arena, tag, fields);

            Ok(term)
        }
        // Case
        9 => {
            let constr = decode_term(ctx, decoder)?;
            let branches = decoder.list_with(ctx, decode_term)?;

            Ok(Term::case(ctx.arena, constr, branches))
        }
        _ => Err(FlatDecodeError::UnknownTermConstructor(tag)),
    }
}

// BLS literals not supported
fn decode_constant<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<&'a Constant<'a>, FlatDecodeError> {
    let tags = decode_constant_tags(ctx, d)?;

    match &tags.as_slice() {
        [0] => {
            let v = d.integer()?;

            Ok(Constant::integer_from(ctx.arena, v))
        }
        [1] => {
            let b = d.bytes(ctx.arena)?;

            Ok(Constant::byte_string(ctx.arena, b))
        }
        [2] => {
            let utf8_bytes = d.bytes(ctx.arena)?;

            let s = BumpString::from_utf8(utf8_bytes)
                .map_err(|e| FlatDecodeError::DecodeUtf8(e.utf8_error()))?;

            Ok(Constant::string(ctx.arena, s))
        }
        [3] => Ok(Constant::unit(ctx.arena)),
        [4] => {
            let v = d.bit()?;

            Ok(Constant::bool(ctx.arena, v))
        }
        [7, 5, rest @ ..] => todo!("list"),

        [7, 7, 6, rest @ ..] => todo!("pair"),

        [8] => {
            let cbor = d.bytes(ctx.arena)?;

            let data = minicbor::decode_with(&cbor, ctx)?;

            Ok(Constant::data(ctx.arena, data))
        }

        x => Err(FlatDecodeError::UnknownConstantConstructor(x.to_vec())),
    }
}

fn decode_constant_tags<'a>(
    ctx: &mut Ctx<'a>,
    d: &mut Decoder,
) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
    d.list_with(ctx, |_arena, d| decode_constant_tag(d))
}

fn decode_constant_tag(d: &mut Decoder) -> Result<u8, FlatDecodeError> {
    d.bits8(CONST_TAG_WIDTH)
}

fn try_from_builtin_tag(arena: &Bump, v: u8) -> Result<&DefaultFunction, FlatDecodeError> {
    match v {
        v if v == DefaultFunction::AddInteger as u8 => Ok(arena.alloc(DefaultFunction::AddInteger)),
        v if v == DefaultFunction::SubtractInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::SubtractInteger))
        }
        v if v == DefaultFunction::MultiplyInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::MultiplyInteger))
        }
        v if v == DefaultFunction::DivideInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::DivideInteger))
        }
        v if v == DefaultFunction::QuotientInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::QuotientInteger))
        }
        v if v == DefaultFunction::RemainderInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::RemainderInteger))
        }
        v if v == DefaultFunction::ModInteger as u8 => Ok(arena.alloc(DefaultFunction::ModInteger)),
        v if v == DefaultFunction::EqualsInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::EqualsInteger))
        }
        v if v == DefaultFunction::LessThanInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::LessThanInteger))
        }
        v if v == DefaultFunction::LessThanEqualsInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::LessThanEqualsInteger))
        }
        // ByteString functions
        v if v == DefaultFunction::AppendByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::AppendByteString))
        }
        v if v == DefaultFunction::ConsByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::ConsByteString))
        }
        v if v == DefaultFunction::SliceByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::SliceByteString))
        }
        v if v == DefaultFunction::LengthOfByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::LengthOfByteString))
        }
        v if v == DefaultFunction::IndexByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::IndexByteString))
        }
        v if v == DefaultFunction::EqualsByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::EqualsByteString))
        }
        v if v == DefaultFunction::LessThanByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::LessThanByteString))
        }
        v if v == DefaultFunction::LessThanEqualsByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::LessThanEqualsByteString))
        }
        // Cryptography and hash functions
        v if v == DefaultFunction::Sha2_256 as u8 => Ok(arena.alloc(DefaultFunction::Sha2_256)),
        v if v == DefaultFunction::Sha3_256 as u8 => Ok(arena.alloc(DefaultFunction::Sha3_256)),
        v if v == DefaultFunction::Blake2b_256 as u8 => {
            Ok(arena.alloc(DefaultFunction::Blake2b_256))
        }
        v if v == DefaultFunction::Blake2b_224 as u8 => {
            Ok(arena.alloc(DefaultFunction::Blake2b_224))
        }
        v if v == DefaultFunction::Keccak_256 as u8 => Ok(arena.alloc(DefaultFunction::Keccak_256)),
        v if v == DefaultFunction::VerifyEd25519Signature as u8 => {
            Ok(arena.alloc(DefaultFunction::VerifyEd25519Signature))
        }
        v if v == DefaultFunction::VerifyEcdsaSecp256k1Signature as u8 => {
            Ok(arena.alloc(DefaultFunction::VerifyEcdsaSecp256k1Signature))
        }
        v if v == DefaultFunction::VerifySchnorrSecp256k1Signature as u8 => {
            Ok(arena.alloc(DefaultFunction::VerifySchnorrSecp256k1Signature))
        }
        // String functions
        v if v == DefaultFunction::AppendString as u8 => {
            Ok(arena.alloc(DefaultFunction::AppendString))
        }
        v if v == DefaultFunction::EqualsString as u8 => {
            Ok(arena.alloc(DefaultFunction::EqualsString))
        }
        v if v == DefaultFunction::EncodeUtf8 as u8 => Ok(arena.alloc(DefaultFunction::EncodeUtf8)),
        v if v == DefaultFunction::DecodeUtf8 as u8 => Ok(arena.alloc(DefaultFunction::DecodeUtf8)),
        // Bool function
        v if v == DefaultFunction::IfThenElse as u8 => Ok(arena.alloc(DefaultFunction::IfThenElse)),
        // Unit function
        v if v == DefaultFunction::ChooseUnit as u8 => Ok(arena.alloc(DefaultFunction::ChooseUnit)),
        // Tracing function
        v if v == DefaultFunction::Trace as u8 => Ok(arena.alloc(DefaultFunction::Trace)),
        // Pairs functions
        v if v == DefaultFunction::FstPair as u8 => Ok(arena.alloc(DefaultFunction::FstPair)),
        v if v == DefaultFunction::SndPair as u8 => Ok(arena.alloc(DefaultFunction::SndPair)),
        // List functions
        v if v == DefaultFunction::ChooseList as u8 => Ok(arena.alloc(DefaultFunction::ChooseList)),
        v if v == DefaultFunction::MkCons as u8 => Ok(arena.alloc(DefaultFunction::MkCons)),
        v if v == DefaultFunction::HeadList as u8 => Ok(arena.alloc(DefaultFunction::HeadList)),
        v if v == DefaultFunction::TailList as u8 => Ok(arena.alloc(DefaultFunction::TailList)),
        v if v == DefaultFunction::NullList as u8 => Ok(arena.alloc(DefaultFunction::NullList)),
        // Data functions
        // It is convenient to have a "choosing" function for a data type that has more than two
        // constructors to get pattern matching over it and we may end up having multiple such data
        // types, hence we include the name of the data type as a suffix.
        v if v == DefaultFunction::ChooseData as u8 => Ok(arena.alloc(DefaultFunction::ChooseData)),
        v if v == DefaultFunction::ConstrData as u8 => Ok(arena.alloc(DefaultFunction::ConstrData)),
        v if v == DefaultFunction::MapData as u8 => Ok(arena.alloc(DefaultFunction::MapData)),
        v if v == DefaultFunction::ListData as u8 => Ok(arena.alloc(DefaultFunction::ListData)),
        v if v == DefaultFunction::IData as u8 => Ok(arena.alloc(DefaultFunction::IData)),
        v if v == DefaultFunction::BData as u8 => Ok(arena.alloc(DefaultFunction::BData)),
        v if v == DefaultFunction::UnConstrData as u8 => {
            Ok(arena.alloc(DefaultFunction::UnConstrData))
        }
        v if v == DefaultFunction::UnMapData as u8 => Ok(arena.alloc(DefaultFunction::UnMapData)),
        v if v == DefaultFunction::UnListData as u8 => Ok(arena.alloc(DefaultFunction::UnListData)),
        v if v == DefaultFunction::UnIData as u8 => Ok(arena.alloc(DefaultFunction::UnIData)),
        v if v == DefaultFunction::UnBData as u8 => Ok(arena.alloc(DefaultFunction::UnBData)),
        v if v == DefaultFunction::EqualsData as u8 => Ok(arena.alloc(DefaultFunction::EqualsData)),
        v if v == DefaultFunction::SerialiseData as u8 => {
            Ok(arena.alloc(DefaultFunction::SerialiseData))
        }
        // Misc constructors
        // Constructors that we need for constructing e.g. Data. Polymorphic builtin
        // constructors are often problematic (See note [Representable built-in
        // functions over polymorphic built-in types])
        v if v == DefaultFunction::MkPairData as u8 => Ok(arena.alloc(DefaultFunction::MkPairData)),
        v if v == DefaultFunction::MkNilData as u8 => Ok(arena.alloc(DefaultFunction::MkNilData)),
        v if v == DefaultFunction::MkNilPairData as u8 => {
            Ok(arena.alloc(DefaultFunction::MkNilPairData))
        }
        v if v == DefaultFunction::Bls12_381_G1_Add as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_Add))
        }
        v if v == DefaultFunction::Bls12_381_G1_Neg as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_Neg))
        }
        v if v == DefaultFunction::Bls12_381_G1_ScalarMul as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_ScalarMul))
        }
        v if v == DefaultFunction::Bls12_381_G1_Equal as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_Equal))
        }
        v if v == DefaultFunction::Bls12_381_G1_Compress as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_Compress))
        }
        v if v == DefaultFunction::Bls12_381_G1_Uncompress as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_Uncompress))
        }
        v if v == DefaultFunction::Bls12_381_G1_HashToGroup as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G1_HashToGroup))
        }
        v if v == DefaultFunction::Bls12_381_G2_Add as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_Add))
        }
        v if v == DefaultFunction::Bls12_381_G2_Neg as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_Neg))
        }
        v if v == DefaultFunction::Bls12_381_G2_ScalarMul as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_ScalarMul))
        }
        v if v == DefaultFunction::Bls12_381_G2_Equal as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_Equal))
        }
        v if v == DefaultFunction::Bls12_381_G2_Compress as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_Compress))
        }
        v if v == DefaultFunction::Bls12_381_G2_Uncompress as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_Uncompress))
        }
        v if v == DefaultFunction::Bls12_381_G2_HashToGroup as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_G2_HashToGroup))
        }
        v if v == DefaultFunction::Bls12_381_MillerLoop as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_MillerLoop))
        }
        v if v == DefaultFunction::Bls12_381_MulMlResult as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_MulMlResult))
        }
        v if v == DefaultFunction::Bls12_381_FinalVerify as u8 => {
            Ok(arena.alloc(DefaultFunction::Bls12_381_FinalVerify))
        }

        // Bitwise
        v if v == DefaultFunction::IntegerToByteString as u8 => {
            Ok(arena.alloc(DefaultFunction::IntegerToByteString))
        }
        v if v == DefaultFunction::ByteStringToInteger as u8 => {
            Ok(arena.alloc(DefaultFunction::ByteStringToInteger))
        }

        _ => Err(FlatDecodeError::DefaultFunctionNotFound(v)),
    }
}
