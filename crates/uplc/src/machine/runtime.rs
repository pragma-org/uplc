use std::array::TryFromSliceError;

use bumpalo::{
    collections::{CollectIn, Vec as BumpVec},
    Bump,
};
use rug::{ops::AddFrom, Assign, Complete};

use crate::{
    builtin::DefaultFunction,
    constant::{self, Constant},
    data::PlutusData,
    typ::Type,
};

use super::{cost_model::builtin_costs::BuiltinCosts, value::Value, ExBudget, MachineError};

#[derive(Debug)]
pub struct Runtime<'a> {
    pub args: BumpVec<'a, &'a Value<'a>>,
    pub fun: &'a DefaultFunction,
    pub forces: usize,
}

impl<'a> Runtime<'a> {
    pub fn new(arena: &'a Bump, fun: &'a DefaultFunction) -> &'a Self {
        arena.alloc(Self {
            args: BumpVec::new_in(arena),
            fun,
            forces: 0,
        })
    }

    pub fn force(&self, arena: &'a Bump) -> &'a Self {
        let new_runtime = arena.alloc(Runtime {
            args: self.args.clone(),
            fun: self.fun,
            forces: self.forces + 1,
        });

        new_runtime
    }

    pub fn push(&self, arena: &'a Bump, arg: &'a Value<'a>) -> &'a Self {
        let new_runtime = arena.alloc(Runtime {
            args: self.args.clone(),
            fun: self.fun,
            forces: self.forces,
        });

        new_runtime.args.push(arg);

        new_runtime
    }

    pub fn needs_force(&self) -> bool {
        self.forces < self.fun.force_count()
    }

    pub fn is_arrow(&self) -> bool {
        self.args.len() < self.fun.arity()
    }

    pub fn is_ready(&self) -> bool {
        self.args.len() == self.fun.arity()
    }

    pub fn to_ex_budget(&self, builtin_costs: &BuiltinCosts) -> ExBudget {
        todo!()
    }

    pub fn call(&self, arena: &'a Bump) -> Result<&'a Value<'a>, MachineError<'a>> {
        match self.fun {
            DefaultFunction::AddInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 + arg2;

                let new = constant::integer(arena);

                new.assign(result);

                let value = Value::integer(arena, new);

                Ok(value)
            }
            DefaultFunction::SubtractInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 - arg2;

                let new = constant::integer(arena);

                new.assign(result);

                let value = Value::integer(arena, new);

                Ok(value)
            }
            DefaultFunction::EqualsInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 == arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::LessThanEqualsInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 <= arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::AppendByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;
                let arg2 = self.args[1].unwrap_byte_string()?;

                let mut result = BumpVec::with_capacity_in(arg1.len() + arg2.len(), arena);

                result.extend_from_slice(arg1);
                result.extend_from_slice(arg2);

                let value = Value::byte_string(arena, result);

                Ok(value)
            }
            DefaultFunction::EqualsByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;
                let arg2 = self.args[1].unwrap_byte_string()?;

                let result = arg1 == arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::IfThenElse => {
                let arg1 = self.args[0].unwrap_bool()?;
                let arg2 = self.args[1];
                let arg3 = self.args[2];

                if arg1 {
                    Ok(arg2)
                } else {
                    Ok(arg3)
                }
            }
            DefaultFunction::MultiplyInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 * arg2;

                let new = constant::integer(arena);

                new.assign(result);

                let value = Value::integer(arena, new);

                Ok(value)
            }
            DefaultFunction::DivideInteger => todo!(),
            DefaultFunction::QuotientInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                if !arg2.is_zero() {
                    let computation = arg1.div_rem_ref(arg2);

                    let q = constant::integer(arena);
                    let r = constant::integer(arena);

                    let mut result = (q, r);

                    result.assign(computation);

                    let value = Value::integer(arena, result.0);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::RemainderInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                if !arg2.is_zero() {
                    let computation = arg1.div_rem_ref(arg2);

                    let q = constant::integer(arena);
                    let r = constant::integer(arena);

                    let mut result = (q, r);

                    result.assign(computation);

                    let value = Value::integer(arena, result.1);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::ModInteger => todo!(),
            DefaultFunction::LessThanInteger => {
                let arg1 = self.args[0].unwrap_integer()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let result = arg1 < arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::ConsByteString => todo!(),
            DefaultFunction::SliceByteString => todo!(),
            DefaultFunction::LengthOfByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;

                let result = arg1.len();

                let new = constant::integer(arena);

                new.assign(result as i64);

                let value = Value::integer(arena, new);

                Ok(value)
            }
            DefaultFunction::IndexByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;
                let arg2 = self.args[1].unwrap_integer()?;

                let index: i128 = arg2.try_into().unwrap();

                if 0 <= index && (index as usize) < arg1.len() {
                    let result = arg1[index as usize];

                    let new = constant::integer(arena);

                    new.assign(result as i64);

                    let value = Value::integer(arena, new);

                    Ok(value)
                } else {
                    Err(MachineError::byte_string_out_of_bounds(arg1, arg2))
                }
            }
            DefaultFunction::LessThanByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;
                let arg2 = self.args[1].unwrap_byte_string()?;

                let result = arg1 < arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::LessThanEqualsByteString => {
                let arg1 = self.args[0].unwrap_byte_string()?;
                let arg2 = self.args[1].unwrap_byte_string()?;

                let result = arg1 <= arg2;

                let value = Value::bool(arena, result);

                Ok(value)
            }
            DefaultFunction::Sha2_256 => {
                use cryptoxide::{digest::Digest, sha2::Sha256};

                let arg1 = self.args[0].unwrap_byte_string()?;

                let mut hasher = Sha256::new();

                hasher.input(arg1);

                let mut bytes = BumpVec::with_capacity_in(hasher.output_bytes(), arena);

                unsafe {
                    bytes.set_len(hasher.output_bytes());
                }

                hasher.result(&mut bytes);

                let value = Value::byte_string(arena, bytes);

                Ok(value)
            }
            DefaultFunction::Sha3_256 => todo!(),
            DefaultFunction::Blake2b_256 => todo!(),
            DefaultFunction::Keccak_256 => todo!(),
            DefaultFunction::Blake2b_224 => todo!(),
            DefaultFunction::VerifyEd25519Signature => {
                use cryptoxide::ed25519;

                let public_key = self.args[0].unwrap_byte_string()?;
                let message = self.args[1].unwrap_byte_string()?;
                let signature = self.args[2].unwrap_byte_string()?;

                let public_key: [u8; 32] =
                    public_key
                        .as_slice()
                        .try_into()
                        .map_err(|e: TryFromSliceError| {
                            MachineError::unexpected_ed25519_public_key_length(e)
                        })?;

                let signature: [u8; 64] =
                    signature
                        .as_slice()
                        .try_into()
                        .map_err(|e: TryFromSliceError| {
                            MachineError::unexpected_ed25519_signature_length(e)
                        })?;

                let valid = ed25519::verify(message, &public_key, &signature);

                let value = Value::bool(arena, valid);

                Ok(value)
            }
            DefaultFunction::VerifyEcdsaSecp256k1Signature => todo!(),
            DefaultFunction::VerifySchnorrSecp256k1Signature => todo!(),
            DefaultFunction::AppendString => todo!(),
            DefaultFunction::EqualsString => todo!(),
            DefaultFunction::EncodeUtf8 => todo!(),
            DefaultFunction::DecodeUtf8 => todo!(),
            DefaultFunction::ChooseUnit => todo!(),
            DefaultFunction::Trace => todo!(),
            DefaultFunction::FstPair => {
                let (_, _, first, _) = self.args[0].unwrap_pair()?;

                let value = Value::con(arena, first);

                Ok(value)
            }
            DefaultFunction::SndPair => {
                let (_, _, _, second) = self.args[0].unwrap_pair()?;

                let value = Value::con(arena, second);

                Ok(value)
            }
            DefaultFunction::ChooseList => {
                let (_, list) = self.args[0].unwrap_list()?;

                if list.is_empty() {
                    Ok(self.args[1])
                } else {
                    Ok(self.args[2])
                }
            }
            DefaultFunction::MkCons => {
                let item = self.args[0].unwrap_constant()?;
                let (typ, list) = self.args[1].unwrap_list()?;

                if item.type_of(arena) != typ {
                    return Err(MachineError::mk_cons_type_mismatch(item));
                }

                let mut new_list = BumpVec::with_capacity_in(list.len() + 1, arena);

                new_list.push(item);

                new_list.extend_from_slice(list);

                let constant = Constant::proto_list(arena, typ, new_list);

                let value = constant.value(arena);

                Ok(value)
            }
            DefaultFunction::HeadList => {
                let (_, list) = self.args[0].unwrap_list()?;

                if list.is_empty() {
                    Err(MachineError::empty_list(list))
                } else {
                    let value = Value::con(arena, list[0]);

                    Ok(value)
                }
            }
            DefaultFunction::TailList => {
                let (t1, list) = self.args[0].unwrap_list()?;

                if list.is_empty() {
                    Err(MachineError::empty_list(list))
                } else {
                    let mut tail = BumpVec::with_capacity_in(list.len(), arena);

                    tail.extend_from_slice(&list[1..]);

                    let constant = Constant::proto_list(arena, t1, tail);

                    let value = Value::con(arena, constant);

                    Ok(value)
                }
            }
            DefaultFunction::NullList => todo!(),
            DefaultFunction::ChooseData => {
                let con = self.args[0].unwrap_constant()?.unwrap_data()?;

                match con {
                    PlutusData::Constr { .. } => Ok(self.args[1]),
                    PlutusData::Map(_) => Ok(self.args[2]),
                    PlutusData::List(_) => Ok(self.args[3]),
                    PlutusData::Integer(_) => Ok(self.args[4]),
                    PlutusData::ByteString(_) => Ok(self.args[5]),
                }
            }
            DefaultFunction::ConstrData => {
                let tag = self.args[0].unwrap_integer()?;
                let (typ, fields) = self.args[1].unwrap_list()?;

                if *typ != Type::Data {
                    return Err(MachineError::type_mismatch(
                        Type::Data,
                        self.args[1].unwrap_constant()?,
                    ));
                }

                let tag = tag.try_into().expect("should cast to u64 just fine");
                let fields = fields
                    .iter()
                    .map(|d| match d {
                        Constant::Data(d) => *d,
                        _ => unreachable!(),
                    })
                    .collect_in(arena);

                let data = PlutusData::constr(arena, tag, fields);

                let constant = Constant::data(arena, data);

                let value = Value::con(arena, constant);

                Ok(value)
            }
            DefaultFunction::MapData => todo!(),
            DefaultFunction::ListData => {
                let (typ, fields) = self.args[0].unwrap_list()?;

                if *typ != Type::Data {
                    return Err(MachineError::type_mismatch(
                        Type::Data,
                        self.args[0].unwrap_constant()?,
                    ));
                }

                let fields = fields
                    .iter()
                    .map(|d| match d {
                        Constant::Data(d) => *d,
                        _ => unreachable!(),
                    })
                    .collect_in(arena);

                let value = PlutusData::list(arena, fields).constant(arena).value(arena);

                Ok(value)
            }
            DefaultFunction::IData => {
                let i = self.args[0].unwrap_integer()?;
                let i = PlutusData::integer(arena, i);

                let value = i.constant(arena).value(arena);

                Ok(value)
            }
            DefaultFunction::BData => {
                let b = self.args[0].unwrap_byte_string()?;

                let b = PlutusData::byte_string(arena, b.clone());

                let value = b.constant(arena).value(arena);

                Ok(value)
            }
            DefaultFunction::UnConstrData => {
                let (tag, fields) = self.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_constr()?;

                let constant = Constant::proto_pair(
                    arena,
                    Type::integer(arena),
                    Type::list(arena, Type::data(arena)),
                    Constant::integer_from(arena, *tag as i128),
                    Constant::proto_list(
                        arena,
                        Type::data(arena),
                        fields
                            .iter()
                            .map(|d| Constant::data(arena, d))
                            .collect_in(arena),
                    ),
                );

                let value = Value::con(arena, constant);

                Ok(value)
            }
            DefaultFunction::UnMapData => todo!(),
            DefaultFunction::UnListData => {
                let list = self.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_list()?;

                let constant = Constant::proto_list(
                    arena,
                    Type::data(arena),
                    list.iter()
                        .map(|d| Constant::data(arena, d))
                        .collect_in(arena),
                );

                let value = Value::con(arena, constant);

                Ok(value)
            }
            DefaultFunction::UnIData => {
                let i = self.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_integer()?;

                let value = Value::integer(arena, i);

                Ok(value)
            }
            DefaultFunction::UnBData => {
                let bs = self.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_byte_string()?;

                let value = Value::byte_string(arena, bs.clone());

                Ok(value)
            }
            DefaultFunction::EqualsData => {
                let d1 = self.args[0].unwrap_constant()?.unwrap_data()?;
                let d2 = self.args[1].unwrap_constant()?.unwrap_data()?;

                let value = Value::bool(arena, d1.eq(d2));

                Ok(value)
            }
            DefaultFunction::SerialiseData => todo!(),
            DefaultFunction::MkPairData => todo!(),
            DefaultFunction::MkNilData => {
                self.args[0].unwrap_unit()?;

                let constant =
                    Constant::proto_list(arena, Type::data(arena), BumpVec::new_in(arena));

                let value = Value::con(arena, constant);

                Ok(value)
            }
            DefaultFunction::MkNilPairData => todo!(),
            DefaultFunction::Bls12_381_G1_Add => todo!(),
            DefaultFunction::Bls12_381_G1_Neg => todo!(),
            DefaultFunction::Bls12_381_G1_ScalarMul => todo!(),
            DefaultFunction::Bls12_381_G1_Equal => todo!(),
            DefaultFunction::Bls12_381_G1_Compress => todo!(),
            DefaultFunction::Bls12_381_G1_Uncompress => todo!(),
            DefaultFunction::Bls12_381_G1_HashToGroup => todo!(),
            DefaultFunction::Bls12_381_G2_Add => todo!(),
            DefaultFunction::Bls12_381_G2_Neg => todo!(),
            DefaultFunction::Bls12_381_G2_ScalarMul => todo!(),
            DefaultFunction::Bls12_381_G2_Equal => todo!(),
            DefaultFunction::Bls12_381_G2_Compress => todo!(),
            DefaultFunction::Bls12_381_G2_Uncompress => todo!(),
            DefaultFunction::Bls12_381_G2_HashToGroup => todo!(),
            DefaultFunction::Bls12_381_MillerLoop => todo!(),
            DefaultFunction::Bls12_381_MulMlResult => todo!(),
            DefaultFunction::Bls12_381_FinalVerify => todo!(),
            DefaultFunction::IntegerToByteString => todo!(),
            DefaultFunction::ByteStringToInteger => todo!(),
        }
    }
}
