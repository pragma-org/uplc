use std::array::TryFromSliceError;

use bumpalo::{
    collections::{CollectIn, String as BumpString, Vec as BumpVec},
    Bump,
};
use rug::Assign;

use crate::{
    builtin::DefaultFunction,
    constant::{self, Constant},
    data::PlutusData,
    typ::Type,
};

use super::{cost_model, value::Value, ExBudget, Machine, MachineError};

pub enum BuiltinSemantics {
    V1,
    V2,
}

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
}

impl<'a> Machine<'a> {
    pub fn call(&mut self, runtime: &'a Runtime<'a>) -> Result<&'a Value<'a>, MachineError<'a>> {
        match runtime.fun {
            DefaultFunction::AddInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.add_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 + arg2;

                let new = constant::integer(self.arena);

                new.assign(result);

                let value = Value::integer(self.arena, new);

                Ok(value)
            }
            DefaultFunction::SubtractInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.subtract_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 - arg2;

                let new = constant::integer(self.arena);

                new.assign(result);

                let value = Value::integer(self.arena, new);

                Ok(value)
            }
            DefaultFunction::EqualsInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.equals_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 == arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::LessThanEqualsInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.less_than_equals_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 <= arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::AppendByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;
                let arg2 = runtime.args[1].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.append_byte_string([
                    cost_model::byte_string_ex_mem(arg1),
                    cost_model::byte_string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let mut result = BumpVec::with_capacity_in(arg1.len() + arg2.len(), self.arena);

                result.extend_from_slice(arg1);
                result.extend_from_slice(arg2);

                let value = Value::byte_string(self.arena, result);

                Ok(value)
            }
            DefaultFunction::EqualsByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;
                let arg2 = runtime.args[1].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.equals_byte_string([
                    cost_model::byte_string_ex_mem(arg1),
                    cost_model::byte_string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 == arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::IfThenElse => {
                let arg1 = runtime.args[0].unwrap_bool()?;
                let arg2 = runtime.args[1];
                let arg3 = runtime.args[2];

                if arg1 {
                    Ok(arg2)
                } else {
                    Ok(arg3)
                }
            }
            DefaultFunction::MultiplyInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.multiply_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 * arg2;

                let new = constant::integer(self.arena);

                new.assign(result);

                let value = Value::integer(self.arena, new);

                Ok(value)
            }
            DefaultFunction::DivideInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.divide_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                if !arg2.is_zero() {
                    let result = arg1 / arg2;

                    let new = constant::integer(self.arena);

                    new.assign(result);

                    let value = Value::integer(self.arena, new);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::QuotientInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.quotient_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                if !arg2.is_zero() {
                    let computation = arg1.div_rem_ref(arg2);

                    let q = constant::integer(self.arena);
                    let r = constant::integer(self.arena);

                    let mut result = (q, r);

                    result.assign(computation);

                    let value = Value::integer(self.arena, result.0);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::RemainderInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.remainder_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                if !arg2.is_zero() {
                    let computation = arg1.div_rem_ref(arg2);

                    let q = constant::integer(self.arena);
                    let r = constant::integer(self.arena);

                    let mut result = (q, r);

                    result.assign(computation);

                    let value = Value::integer(self.arena, result.1);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::ModInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.mod_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                if !arg2.is_zero() {
                    let result = constant::integer(self.arena);

                    let computation = arg1.modulo_ref(arg2);

                    result.assign(computation);

                    let value = Value::integer(self.arena, result);

                    Ok(value)
                } else {
                    Err(MachineError::division_by_zero(arg1, arg2))
                }
            }
            DefaultFunction::LessThanInteger => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.less_than_integer([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 < arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::ConsByteString => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.cons_byte_string([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::byte_string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let byte: u8 = match &self.semantics {
                    BuiltinSemantics::V1 => {
                        let wrap = constant::integer(self.arena);

                        let max = constant::integer_from(self.arena, 256);

                        wrap.assign(arg1.modulo_ref(max));

                        (&*wrap).try_into().expect("should cast to u64 just fine")
                    }
                    BuiltinSemantics::V2 => {
                        if *arg1 > 255 || *arg1 < 0 {
                            return Err(MachineError::byte_string_cons_not_a_byte(arg1));
                        }

                        arg1.try_into().expect("should cast to u8 just fine")
                    }
                };

                let mut ret = BumpVec::with_capacity_in(arg2.len() + 1, self.arena);

                ret.push(byte);

                ret.extend_from_slice(arg2);

                let value = Value::byte_string(self.arena, ret);

                Ok(value)
            }
            DefaultFunction::SliceByteString => {
                let arg1 = runtime.args[0].unwrap_integer()?;
                let arg2 = runtime.args[1].unwrap_integer()?;
                let arg3 = runtime.args[2].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.slice_byte_string([
                    cost_model::integer_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                    cost_model::byte_string_ex_mem(arg3),
                ]);

                self.spend_budget(budget)?;

                let skip: usize = if *arg1 < 0 {
                    0
                } else {
                    arg1.try_into().expect("should cast to usize just fine")
                };

                let take: usize = if *arg2 < 0 {
                    0
                } else {
                    arg2.try_into().expect("should cast to usize just fine")
                };

                let ret = arg3
                    .iter()
                    .skip(skip)
                    .take(take)
                    .cloned()
                    .collect_in(self.arena);

                let value = Value::byte_string(self.arena, ret);

                Ok(value)
            }
            DefaultFunction::LengthOfByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .length_of_byte_string([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let result = arg1.len();

                let new = constant::integer(self.arena);

                new.assign(result as i64);

                let value = Value::integer(self.arena, new);

                Ok(value)
            }
            DefaultFunction::IndexByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;
                let arg2 = runtime.args[1].unwrap_integer()?;

                let budget = self.costs.builtin_costs.index_byte_string([
                    cost_model::byte_string_ex_mem(arg1),
                    cost_model::integer_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let index: i128 = arg2.try_into().unwrap();

                if 0 <= index && (index as usize) < arg1.len() {
                    let result = arg1[index as usize];

                    let new = constant::integer(self.arena);

                    new.assign(result as i64);

                    let value = Value::integer(self.arena, new);

                    Ok(value)
                } else {
                    Err(MachineError::byte_string_out_of_bounds(arg1, arg2))
                }
            }
            DefaultFunction::LessThanByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;
                let arg2 = runtime.args[1].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.less_than_byte_string([
                    cost_model::byte_string_ex_mem(arg1),
                    cost_model::byte_string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 < arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::LessThanEqualsByteString => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;
                let arg2 = runtime.args[1].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.less_than_equals_byte_string([
                    cost_model::byte_string_ex_mem(arg1),
                    cost_model::byte_string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let result = arg1 <= arg2;

                let value = Value::bool(self.arena, result);

                Ok(value)
            }
            DefaultFunction::Sha2_256 => {
                use cryptoxide::{digest::Digest, sha2::Sha256};

                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .sha2_256([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let mut hasher = Sha256::new();

                hasher.input(arg1);

                let mut bytes = BumpVec::with_capacity_in(hasher.output_bytes(), self.arena);

                unsafe {
                    bytes.set_len(hasher.output_bytes());
                }

                hasher.result(&mut bytes);

                let value = Value::byte_string(self.arena, bytes);

                Ok(value)
            }
            DefaultFunction::Sha3_256 => {
                use cryptoxide::{digest::Digest, sha3::Sha3_256};

                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .sha3_256([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let mut hasher = Sha3_256::new();

                hasher.input(arg1);

                let mut bytes = BumpVec::with_capacity_in(hasher.output_bytes(), self.arena);

                unsafe {
                    bytes.set_len(hasher.output_bytes());
                }

                hasher.result(&mut bytes);

                let value = Value::byte_string(self.arena, bytes);

                Ok(value)
            }
            DefaultFunction::Blake2b_256 => {
                use cryptoxide::{blake2b::Blake2b, digest::Digest};

                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .blake2b_256([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let mut digest = BumpVec::with_capacity_in(32, self.arena);

                unsafe {
                    digest.set_len(32);
                }

                let mut context = Blake2b::new(32);

                context.input(arg1);
                context.result(&mut digest);

                let value = Value::byte_string(self.arena, digest);

                Ok(value)
            }
            DefaultFunction::Keccak_256 => {
                use cryptoxide::{digest::Digest, sha3::Keccak256};

                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .keccak_256([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let mut hasher = Keccak256::new();

                hasher.input(arg1);

                let mut bytes = BumpVec::with_capacity_in(hasher.output_bytes(), self.arena);

                unsafe {
                    bytes.set_len(hasher.output_bytes());
                }

                hasher.result(&mut bytes);

                let value = Value::byte_string(self.arena, bytes);

                Ok(value)
            }
            DefaultFunction::Blake2b_224 => {
                use cryptoxide::{blake2b::Blake2b, digest::Digest};

                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .blake2b_224([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let mut digest = BumpVec::with_capacity_in(28, self.arena);

                unsafe {
                    digest.set_len(28);
                }

                let mut context = Blake2b::new(28);

                context.input(arg1);
                context.result(&mut digest);

                let value = Value::byte_string(self.arena, digest);

                Ok(value)
            }
            DefaultFunction::VerifyEd25519Signature => {
                use cryptoxide::ed25519;

                let public_key = runtime.args[0].unwrap_byte_string()?;
                let message = runtime.args[1].unwrap_byte_string()?;
                let signature = runtime.args[2].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.verify_ed25519_signature([
                    cost_model::byte_string_ex_mem(public_key),
                    cost_model::byte_string_ex_mem(message),
                    cost_model::byte_string_ex_mem(signature),
                ]);

                self.spend_budget(budget)?;

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

                let value = Value::bool(self.arena, valid);

                Ok(value)
            }
            DefaultFunction::VerifyEcdsaSecp256k1Signature => {
                use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};

                let public_key = runtime.args[0].unwrap_byte_string()?;
                let message = runtime.args[1].unwrap_byte_string()?;
                let signature = runtime.args[2].unwrap_byte_string()?;

                let budget = self.costs.builtin_costs.verify_ecdsa_secp256k1_signature([
                    cost_model::byte_string_ex_mem(public_key),
                    cost_model::byte_string_ex_mem(message),
                    cost_model::byte_string_ex_mem(signature),
                ]);

                self.spend_budget(budget)?;

                let secp = Secp256k1::verification_only();

                let public_key =
                    PublicKey::from_slice(public_key).map_err(MachineError::secp256k1)?;

                let signature =
                    Signature::from_compact(signature).map_err(MachineError::secp256k1)?;

                let message =
                    Message::from_digest_slice(message).map_err(MachineError::secp256k1)?;

                let valid = secp.verify_ecdsa(&message, &signature, &public_key);

                let value = Value::bool(self.arena, valid.is_ok());

                Ok(value)
            }
            DefaultFunction::VerifySchnorrSecp256k1Signature => {
                use secp256k1::{schnorr::Signature, Secp256k1, XOnlyPublicKey};

                let public_key = runtime.args[0].unwrap_byte_string()?;
                let message = runtime.args[1].unwrap_byte_string()?;
                let signature = runtime.args[2].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .verify_schnorr_secp256k1_signature([
                        cost_model::byte_string_ex_mem(public_key),
                        cost_model::byte_string_ex_mem(message),
                        cost_model::byte_string_ex_mem(signature),
                    ]);

                self.spend_budget(budget)?;

                let secp = Secp256k1::verification_only();

                let public_key =
                    XOnlyPublicKey::from_slice(public_key).map_err(MachineError::secp256k1)?;

                let signature =
                    Signature::from_slice(signature).map_err(MachineError::secp256k1)?;

                let valid = secp.verify_schnorr(&signature, message, &public_key);

                let value = Value::bool(self.arena, valid.is_ok());

                Ok(value)
            }
            DefaultFunction::AppendString => todo!(),
            DefaultFunction::EqualsString => {
                let arg1 = runtime.args[0].unwrap_string()?;
                let arg2 = runtime.args[1].unwrap_string()?;

                let budget = self.costs.builtin_costs.equals_string([
                    cost_model::string_ex_mem(arg1),
                    cost_model::string_ex_mem(arg2),
                ]);

                self.spend_budget(budget)?;

                let value = Value::bool(self.arena, arg1 == arg2);

                Ok(value)
            }
            DefaultFunction::EncodeUtf8 => {
                let arg1 = runtime.args[0].unwrap_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .encode_utf8([cost_model::string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let s_bytes = arg1.as_bytes();

                let mut bytes = BumpVec::with_capacity_in(s_bytes.len(), self.arena);

                bytes.extend_from_slice(s_bytes);

                let value = Value::byte_string(self.arena, bytes);

                Ok(value)
            }
            DefaultFunction::DecodeUtf8 => {
                let arg1 = runtime.args[0].unwrap_byte_string()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .decode_utf8([cost_model::byte_string_ex_mem(arg1)]);

                self.spend_budget(budget)?;

                let string = BumpString::from_utf8(arg1.clone())
                    .map_err(|e| MachineError::decode_utf8(e.utf8_error()))?;

                let value = Value::string(self.arena, string);

                Ok(value)
            }
            DefaultFunction::ChooseUnit => {
                runtime.args[0].unwrap_unit()?;
                let arg2 = runtime.args[1];

                let budget = self
                    .costs
                    .builtin_costs
                    .choose_unit([cost_model::UNIT_EX_MEM, cost_model::value_ex_mem(arg2)]);

                self.spend_budget(budget)?;

                Ok(arg2)
            }
            DefaultFunction::Trace => todo!(),
            DefaultFunction::FstPair => {
                let (_, _, first, second) = runtime.args[0].unwrap_pair()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .fst_pair([cost_model::pair_ex_mem(first, second)]);

                self.spend_budget(budget)?;

                let value = Value::con(self.arena, first);

                Ok(value)
            }
            DefaultFunction::SndPair => {
                let (_, _, first, second) = runtime.args[0].unwrap_pair()?;

                let budget = self
                    .costs
                    .builtin_costs
                    .snd_pair([cost_model::pair_ex_mem(first, second)]);

                self.spend_budget(budget)?;

                let value = Value::con(self.arena, second);

                Ok(value)
            }
            DefaultFunction::ChooseList => {
                let (_, list) = runtime.args[0].unwrap_list()?;

                if list.is_empty() {
                    Ok(runtime.args[1])
                } else {
                    Ok(runtime.args[2])
                }
            }
            DefaultFunction::MkCons => {
                let item = runtime.args[0].unwrap_constant()?;
                let (typ, list) = runtime.args[1].unwrap_list()?;

                if item.type_of(self.arena) != typ {
                    return Err(MachineError::mk_cons_type_mismatch(item));
                }

                let mut new_list = BumpVec::with_capacity_in(list.len() + 1, self.arena);

                new_list.push(item);

                new_list.extend_from_slice(list);

                let constant = Constant::proto_list(self.arena, typ, new_list);

                let value = constant.value(self.arena);

                Ok(value)
            }
            DefaultFunction::HeadList => {
                let (_, list) = runtime.args[0].unwrap_list()?;

                if list.is_empty() {
                    Err(MachineError::empty_list(list))
                } else {
                    let value = Value::con(self.arena, list[0]);

                    Ok(value)
                }
            }
            DefaultFunction::TailList => {
                let (t1, list) = runtime.args[0].unwrap_list()?;

                if list.is_empty() {
                    Err(MachineError::empty_list(list))
                } else {
                    let mut tail = BumpVec::with_capacity_in(list.len(), self.arena);

                    tail.extend_from_slice(&list[1..]);

                    let constant = Constant::proto_list(self.arena, t1, tail);

                    let value = Value::con(self.arena, constant);

                    Ok(value)
                }
            }
            DefaultFunction::NullList => {
                let (_, list) = runtime.args[0].unwrap_list()?;

                let value = Value::bool(self.arena, list.is_empty());

                Ok(value)
            }
            DefaultFunction::ChooseData => {
                let con = runtime.args[0].unwrap_constant()?.unwrap_data()?;

                match con {
                    PlutusData::Constr { .. } => Ok(runtime.args[1]),
                    PlutusData::Map(_) => Ok(runtime.args[2]),
                    PlutusData::List(_) => Ok(runtime.args[3]),
                    PlutusData::Integer(_) => Ok(runtime.args[4]),
                    PlutusData::ByteString(_) => Ok(runtime.args[5]),
                }
            }
            DefaultFunction::ConstrData => {
                let tag = runtime.args[0].unwrap_integer()?;
                let (typ, fields) = runtime.args[1].unwrap_list()?;

                if *typ != Type::Data {
                    return Err(MachineError::type_mismatch(
                        Type::Data,
                        runtime.args[1].unwrap_constant()?,
                    ));
                }

                let tag = tag.try_into().expect("should cast to u64 just fine");
                let fields = fields
                    .iter()
                    .map(|d| match d {
                        Constant::Data(d) => *d,
                        _ => unreachable!(),
                    })
                    .collect_in(self.arena);

                let data = PlutusData::constr(self.arena, tag, fields);

                let constant = Constant::data(self.arena, data);

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
            DefaultFunction::MapData => todo!(),
            DefaultFunction::ListData => {
                let (typ, fields) = runtime.args[0].unwrap_list()?;

                if *typ != Type::Data {
                    return Err(MachineError::type_mismatch(
                        Type::Data,
                        runtime.args[0].unwrap_constant()?,
                    ));
                }

                let fields = fields
                    .iter()
                    .map(|d| match d {
                        Constant::Data(d) => *d,
                        _ => unreachable!(),
                    })
                    .collect_in(self.arena);

                let value = PlutusData::list(self.arena, fields)
                    .constant(self.arena)
                    .value(self.arena);

                Ok(value)
            }
            DefaultFunction::IData => {
                let i = runtime.args[0].unwrap_integer()?;
                let i = PlutusData::integer(self.arena, i);

                let value = i.constant(self.arena).value(self.arena);

                Ok(value)
            }
            DefaultFunction::BData => {
                let b = runtime.args[0].unwrap_byte_string()?;

                let b = PlutusData::byte_string(self.arena, b.clone());

                let value = b.constant(self.arena).value(self.arena);

                Ok(value)
            }
            DefaultFunction::UnConstrData => {
                let (tag, fields) = runtime.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_constr()?;

                let constant = Constant::proto_pair(
                    self.arena,
                    Type::integer(self.arena),
                    Type::list(self.arena, Type::data(self.arena)),
                    Constant::integer_from(self.arena, *tag as i128),
                    Constant::proto_list(
                        self.arena,
                        Type::data(self.arena),
                        fields
                            .iter()
                            .map(|d| Constant::data(self.arena, d))
                            .collect_in(self.arena),
                    ),
                );

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
            DefaultFunction::UnMapData => todo!(),
            DefaultFunction::UnListData => {
                let list = runtime.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_list()?;

                let constant = Constant::proto_list(
                    self.arena,
                    Type::data(self.arena),
                    list.iter()
                        .map(|d| Constant::data(self.arena, d))
                        .collect_in(self.arena),
                );

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
            DefaultFunction::UnIData => {
                let i = runtime.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_integer()?;

                let value = Value::integer(self.arena, i);

                Ok(value)
            }
            DefaultFunction::UnBData => {
                let bs = runtime.args[0]
                    .unwrap_constant()?
                    .unwrap_data()?
                    .unwrap_byte_string()?;

                let value = Value::byte_string(self.arena, bs.clone());

                Ok(value)
            }
            DefaultFunction::EqualsData => {
                let d1 = runtime.args[0].unwrap_constant()?.unwrap_data()?;
                let d2 = runtime.args[1].unwrap_constant()?.unwrap_data()?;

                let value = Value::bool(self.arena, d1.eq(d2));

                Ok(value)
            }
            DefaultFunction::SerialiseData => todo!(),
            DefaultFunction::MkPairData => {
                let d1 = runtime.args[0].unwrap_constant()?.unwrap_data()?;
                let d2 = runtime.args[1].unwrap_constant()?.unwrap_data()?;

                let constant = Constant::proto_pair(
                    self.arena,
                    Type::data(self.arena),
                    Type::data(self.arena),
                    Constant::data(self.arena, d1),
                    Constant::data(self.arena, d2),
                );

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
            DefaultFunction::MkNilData => {
                runtime.args[0].unwrap_unit()?;

                let constant = Constant::proto_list(
                    self.arena,
                    Type::data(self.arena),
                    BumpVec::new_in(self.arena),
                );

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
            DefaultFunction::MkNilPairData => {
                runtime.args[0].unwrap_unit()?;

                let constant = Constant::proto_list(
                    self.arena,
                    Type::pair(self.arena, Type::data(self.arena), Type::data(self.arena)),
                    BumpVec::new_in(self.arena),
                );

                let value = Value::con(self.arena, constant);

                Ok(value)
            }
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
