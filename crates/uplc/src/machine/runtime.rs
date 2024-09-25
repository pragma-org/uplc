use bumpalo::{collections::Vec as BumpVec, Bump};
use rug::Assign;

use crate::{
    builtin::DefaultFunction,
    constant::{self},
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
            DefaultFunction::QuotientInteger => todo!(),
            DefaultFunction::RemainderInteger => todo!(),
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
            DefaultFunction::Sha2_256 => todo!(),
            DefaultFunction::Sha3_256 => todo!(),
            DefaultFunction::Blake2b_256 => todo!(),
            DefaultFunction::Keccak_256 => todo!(),
            DefaultFunction::Blake2b_224 => todo!(),
            DefaultFunction::VerifyEd25519Signature => todo!(),
            DefaultFunction::VerifyEcdsaSecp256k1Signature => todo!(),
            DefaultFunction::VerifySchnorrSecp256k1Signature => todo!(),
            DefaultFunction::AppendString => todo!(),
            DefaultFunction::EqualsString => todo!(),
            DefaultFunction::EncodeUtf8 => todo!(),
            DefaultFunction::DecodeUtf8 => todo!(),
            DefaultFunction::ChooseUnit => todo!(),
            DefaultFunction::Trace => todo!(),
            DefaultFunction::FstPair => todo!(),
            DefaultFunction::SndPair => todo!(),
            DefaultFunction::ChooseList => todo!(),
            DefaultFunction::MkCons => todo!(),
            DefaultFunction::HeadList => todo!(),
            DefaultFunction::TailList => todo!(),
            DefaultFunction::NullList => todo!(),
            DefaultFunction::ChooseData => todo!(),
            DefaultFunction::ConstrData => todo!(),
            DefaultFunction::MapData => todo!(),
            DefaultFunction::ListData => todo!(),
            DefaultFunction::IData => todo!(),
            DefaultFunction::BData => todo!(),
            DefaultFunction::UnConstrData => todo!(),
            DefaultFunction::UnMapData => todo!(),
            DefaultFunction::UnListData => todo!(),
            DefaultFunction::UnIData => todo!(),
            DefaultFunction::UnBData => todo!(),
            DefaultFunction::EqualsData => todo!(),
            DefaultFunction::SerialiseData => todo!(),
            DefaultFunction::MkPairData => todo!(),
            DefaultFunction::MkNilData => todo!(),
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
