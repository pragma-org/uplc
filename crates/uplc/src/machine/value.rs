use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};

use crate::{
    constant::{Constant, Integer},
    term::Term,
    typ::Type,
};

use super::{env::Env, runtime::Runtime, MachineError};

#[derive(Debug)]
pub enum Value<'a> {
    Con(&'a Constant<'a>),
    Lambda {
        parameter: usize,
        body: &'a Term<'a>,
        env: &'a Env<'a>,
    },
    Builtin(&'a Runtime<'a>),
    Delay(&'a Term<'a>, &'a Env<'a>),
    Constr(usize, BumpVec<'a, &'a Value<'a>>),
}

impl<'a> Value<'a> {
    pub fn con(arena: &'a Bump, constant: &'a Constant<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Con(constant))
    }

    pub fn lambda(
        arena: &'a Bump,
        parameter: usize,
        body: &'a Term<'a>,
        env: &'a Env<'a>,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Lambda {
            parameter,
            body,
            env,
        })
    }

    pub fn delay(arena: &'a Bump, body: &'a Term<'a>, env: &'a Env<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Delay(body, env))
    }

    pub fn constr_empty(arena: &'a Bump, tag: usize) -> &'a Value<'a> {
        arena.alloc(Value::Constr(tag, BumpVec::new_in(arena)))
    }

    pub fn constr(
        arena: &'a Bump,
        tag: usize,
        values: BumpVec<'a, &'a Value<'a>>,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Constr(tag, values))
    }

    pub fn builtin(arena: &'a Bump, runtime: &'a Runtime<'a>) -> &'a Value<'a> {
        arena.alloc(Value::Builtin(runtime))
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Integer(i));

        Value::con(arena, con)
    }

    pub fn byte_string(arena: &'a Bump, b: BumpVec<'a, u8>) -> &'a Value<'a> {
        let con = arena.alloc(Constant::ByteString(b));

        Value::con(arena, con)
    }

    pub fn string(arena: &'a Bump, s: BumpString<'a>) -> &'a Value<'a> {
        let con = arena.alloc(Constant::String(s));

        Value::con(arena, con)
    }

    pub fn bool(arena: &'a Bump, b: bool) -> &'a Value<'a> {
        let con = arena.alloc(Constant::Boolean(b));

        Value::con(arena, con)
    }

    pub fn unwrap_integer(&'a self) -> Result<&'a Integer, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Integer(integer) = inner else {
            return Err(MachineError::type_mismatch(Type::Integer, inner));
        };

        Ok(integer)
    }

    pub fn unwrap_byte_string(&'a self) -> Result<&BumpVec<'a, u8>, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::ByteString(byte_string) = inner else {
            return Err(MachineError::type_mismatch(Type::ByteString, inner));
        };

        Ok(byte_string)
    }

    pub fn unwrap_string(&'a self) -> Result<&BumpString<'a>, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::String(string) = inner else {
            return Err(MachineError::type_mismatch(Type::String, inner));
        };

        Ok(string)
    }

    pub fn unwrap_bool(&'a self) -> Result<bool, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Boolean(b) = inner else {
            return Err(MachineError::type_mismatch(Type::Bool, inner));
        };

        Ok(*b)
    }

    pub fn unwrap_pair(
        &'a self,
    ) -> Result<
        (
            &'a Type<'a>,
            &'a Type<'a>,
            &'a Constant<'a>,
            &'a Constant<'a>,
        ),
        MachineError<'a>,
    > {
        let inner = self.unwrap_constant()?;

        let Constant::ProtoPair(t1, t2, first, second) = inner else {
            return Err(MachineError::expected_pair(inner));
        };

        Ok((t1, t2, first, second))
    }

    pub fn unwrap_list(
        &'a self,
    ) -> Result<(&'a Type<'a>, &'a BumpVec<'a, &'a Constant<'a>>), MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::ProtoList(t1, list) = inner else {
            return Err(MachineError::expected_list(inner));
        };

        Ok((t1, list))
    }

    pub fn unwrap_map(
        &'a self,
    ) -> Result<(&'a Type<'a>, &'a BumpVec<'a, &'a Constant<'a>>), MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::ProtoList(t1, list) = inner else {
            return Err(MachineError::expected_list(inner));
        };

        Ok((t1, list))
    }

    pub fn unwrap_constant(&'a self) -> Result<&'a Constant<'a>, MachineError<'a>> {
        let Value::Con(item) = self else {
            return Err(MachineError::NotAConstant(self));
        };

        Ok(item)
    }

    pub fn unwrap_unit(&'a self) -> Result<(), MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Unit = inner else {
            return Err(MachineError::type_mismatch(Type::Unit, inner));
        };

        Ok(())
    }

    pub fn unwrap_bls12_381_g1_element(&'a self) -> Result<&'a blst::blst_p1, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Bls12_381G1Element(g1) = inner else {
            return Err(MachineError::type_mismatch(Type::Bls12_381G1Element, inner));
        };

        Ok(g1)
    }

    pub fn unwrap_bls12_381_g2_element(&'a self) -> Result<&'a blst::blst_p2, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Bls12_381G2Element(g2) = inner else {
            return Err(MachineError::type_mismatch(Type::Bls12_381G2Element, inner));
        };

        Ok(g2)
    }

    pub fn unwrap_bls12_381_ml_result(&'a self) -> Result<&'a blst::blst_fp12, MachineError<'a>> {
        let inner = self.unwrap_constant()?;

        let Constant::Bls12_381MlResult(ml_res) = inner else {
            return Err(MachineError::type_mismatch(Type::Bls12_381MlResult, inner));
        };

        Ok(ml_res)
    }
}

impl<'a> Constant<'a> {
    pub fn value(&'a self, arena: &'a Bump) -> &'a Value<'a> {
        Value::con(arena, self)
    }
}
