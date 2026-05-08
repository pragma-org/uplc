//! UPLC term AST and builder helpers.
//!
//! [`Term`] is the central type of this crate. It covers all UPLC term constructors:
//! variables, lambdas, function application, delay/force, constants, built-ins, and the
//! Plutus V3 `Constr`/`Case` constructors. Builder methods allocate into the shared
//! [`Arena`], so all returned references carry the arena lifetime `'a`.

use crate::{
    arena::Arena,
    builtin::DefaultFunction,
    constant::{integer_from, Constant, Integer},
    data::PlutusData,
};

/// A UPLC term.
///
/// All recursive sub-terms are arena-allocated references, making the structure
/// cheap to traverse without heap churn. Builder methods on `Term` allocate into
/// the shared [`Arena`].
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum Term<'a, V> {
    /// A variable reference. `V` is the binder strategy (e.g. [`DeBruijn`](crate::binder::DeBruijn)).
    Var(&'a V),

    /// A lambda abstraction binding one variable.
    Lambda {
        /// The bound variable.
        parameter: &'a V,
        /// The body of the abstraction.
        body: &'a Term<'a, V>,
    },

    /// Function application.
    Apply {
        /// The function term being applied.
        function: &'a Term<'a, V>,
        /// The argument being passed.
        argument: &'a Term<'a, V>,
    },

    /// Wraps a term in a thunk (introduction form for `Force`).
    Delay(&'a Term<'a, V>),

    /// Evaluates a delayed term.
    Force(&'a Term<'a, V>),

    /// Pattern-match on a constructor value (Plutus V3 sums-of-products).
    Case {
        /// The scrutinee constructor term.
        constr: &'a Term<'a, V>,
        /// One branch term per constructor alternative.
        branches: &'a [&'a Term<'a, V>],
    },

    /// Construct a tagged product value (Plutus V3 sums-of-products).
    Constr {
        /// Constructor alternative index.
        tag: usize,
        /// Field values of the constructor.
        fields: &'a [&'a Term<'a, V>],
    },

    /// A literal constant value.
    Constant(&'a Constant<'a>),

    /// A reference to a built-in function.
    Builtin(&'a DefaultFunction),

    /// Explicit error term; always reduces to a runtime error.
    Error,
}

impl<'a, V> Term<'a, V> {
    /// Allocates a [`Term::Var`] referencing `i`.
    pub fn var(arena: &'a Arena, i: &'a V) -> &'a Term<'a, V> {
        arena.alloc(Term::Var(i))
    }

    /// Applies `argument` to `self`, allocating a [`Term::Apply`].
    pub fn apply(&'a self, arena: &'a Arena, argument: &'a Term<'a, V>) -> &'a Term<'a, V> {
        arena.alloc(Term::Apply {
            function: self,
            argument,
        })
    }

    /// Wraps `self` in a [`Term::Lambda`] binding `parameter`.
    pub fn lambda(&'a self, arena: &'a Arena, parameter: &'a V) -> &'a Term<'a, V> {
        arena.alloc(Term::Lambda {
            parameter,
            body: self,
        })
    }

    /// Wraps `self` in a [`Term::Force`].
    pub fn force(&'a self, arena: &'a Arena) -> &'a Term<'a, V> {
        arena.alloc(Term::Force(self))
    }

    /// Wraps `self` in a [`Term::Delay`].
    pub fn delay(&'a self, arena: &'a Arena) -> &'a Term<'a, V> {
        arena.alloc(Term::Delay(self))
    }

    /// Allocates a [`Term::Constant`].
    pub fn constant(arena: &'a Arena, constant: &'a Constant<'a>) -> &'a Term<'a, V> {
        arena.alloc(Term::Constant(constant))
    }

    /// Allocates a [`Term::Constr`] with the given `tag` and `fields` (Plutus V3).
    pub fn constr(arena: &'a Arena, tag: usize, fields: &'a [&'a Term<'a, V>]) -> &'a Term<'a, V> {
        arena.alloc(Term::Constr { tag, fields })
    }

    /// Allocates a [`Term::Case`] over `constr` with the given `branches` (Plutus V3).
    pub fn case(
        arena: &'a Arena,
        constr: &'a Term<'a, V>,
        branches: &'a [&'a Term<'a, V>],
    ) -> &'a Term<'a, V> {
        arena.alloc(Term::Case { constr, branches })
    }

    /// Allocates a constant integer term.
    pub fn integer(arena: &'a Arena, i: &'a Integer) -> &'a Term<'a, V> {
        let constant = arena.alloc(Constant::Integer(i));

        Term::constant(arena, constant)
    }

    /// Allocates a constant integer term from an `i128`.
    pub fn integer_from(arena: &'a Arena, i: i128) -> &'a Term<'a, V> {
        Self::integer(arena, integer_from(arena, i))
    }

    /// Allocates a constant byte-string term.
    pub fn byte_string(arena: &'a Arena, bytes: &'a [u8]) -> &'a Term<'a, V> {
        let constant = Constant::byte_string(arena, bytes);

        Term::constant(arena, constant)
    }

    /// Allocates a constant UTF-8 string term.
    pub fn string(arena: &'a Arena, s: &'a str) -> &'a Term<'a, V> {
        let constant = Constant::string(arena, s);

        Term::constant(arena, constant)
    }

    /// Allocates a constant boolean term.
    pub fn bool(arena: &'a Arena, v: bool) -> &'a Term<'a, V> {
        let constant = Constant::bool(arena, v);

        Term::constant(arena, constant)
    }

    /// Allocates a constant [`PlutusData`] term.
    pub fn data(arena: &'a Arena, d: &'a PlutusData<'a>) -> &'a Term<'a, V> {
        let constant = Constant::data(arena, d);

        Term::constant(arena, constant)
    }

    /// Allocates a constant [`PlutusData::ByteString`] term.
    pub fn data_byte_string(arena: &'a Arena, bytes: &'a [u8]) -> &'a Term<'a, V> {
        let data = PlutusData::byte_string(arena, bytes);

        Term::data(arena, data)
    }

    /// Allocates a constant [`PlutusData::Integer`] term.
    pub fn data_integer(arena: &'a Arena, i: &'a Integer) -> &'a Term<'a, V> {
        let data = PlutusData::integer(arena, i);

        Term::data(arena, data)
    }

    /// Allocates a constant [`PlutusData::Integer`] term from an `i128`.
    pub fn data_integer_from(arena: &'a Arena, i: i128) -> &'a Term<'a, V> {
        let data = PlutusData::integer_from(arena, i);

        Term::data(arena, data)
    }

    /// Allocates a constant unit term.
    pub fn unit(arena: &'a Arena) -> &'a Term<'a, V> {
        let constant = Constant::unit(arena);

        Term::constant(arena, constant)
    }

    /// Allocates a [`Term::Builtin`] for `fun`.
    pub fn builtin(arena: &'a Arena, fun: &'a DefaultFunction) -> &'a Term<'a, V> {
        arena.alloc(Term::Builtin(fun))
    }

    /// Allocates a [`Term::Error`].
    pub fn error(arena: &'a Arena) -> &'a Term<'a, V> {
        arena.alloc(Term::Error)
    }

    // --- Integer built-in shorthands ---

    /// Builtin term for [`DefaultFunction::AddInteger`].
    pub fn add_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::AddInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MultiplyInteger`].
    pub fn multiply_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MultiplyInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::DivideInteger`].
    pub fn divide_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::DivideInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::QuotientInteger`].
    pub fn quotient_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::QuotientInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::RemainderInteger`].
    pub fn remainder_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::RemainderInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ModInteger`].
    pub fn mod_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ModInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::SubtractInteger`].
    pub fn subtract_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::SubtractInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::EqualsInteger`].
    pub fn equals_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::EqualsInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LessThanEqualsInteger`].
    pub fn less_than_equals_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LessThanEqualsInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LessThanInteger`].
    pub fn less_than_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LessThanInteger);

        Term::builtin(arena, fun)
    }

    // --- Control ---

    /// Builtin term for [`DefaultFunction::IfThenElse`].
    pub fn if_then_else(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::IfThenElse);

        Term::builtin(arena, fun)
    }

    // --- ByteString ---

    /// Builtin term for [`DefaultFunction::AppendByteString`].
    pub fn append_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::AppendByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::EqualsByteString`].
    pub fn equals_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::EqualsByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ConsByteString`].
    pub fn cons_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ConsByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::SliceByteString`].
    pub fn slice_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::SliceByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LengthOfByteString`].
    pub fn length_of_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LengthOfByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::IndexByteString`].
    pub fn index_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::IndexByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LessThanByteString`].
    pub fn less_than_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LessThanByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LessThanEqualsByteString`].
    pub fn less_than_equals_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LessThanEqualsByteString);

        Term::builtin(arena, fun)
    }

    // --- Cryptography ---

    /// Builtin term for [`DefaultFunction::Sha2_256`].
    pub fn sha2_256(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Sha2_256);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Sha3_256`].
    pub fn sha3_256(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Sha3_256);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Blake2b_256`].
    pub fn blake2b_256(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Blake2b_256);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Keccak_256`].
    pub fn keccak_256(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Keccak_256);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Blake2b_224`].
    pub fn blake2b_224(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Blake2b_224);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::VerifyEd25519Signature`].
    pub fn verify_ed25519_signature(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::VerifyEd25519Signature);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::VerifyEcdsaSecp256k1Signature`].
    pub fn verify_ecdsa_secp256k1_signature(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::VerifyEcdsaSecp256k1Signature);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::VerifySchnorrSecp256k1Signature`].
    pub fn verify_schnorr_secp256k1_signature(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::VerifySchnorrSecp256k1Signature);

        Term::builtin(arena, fun)
    }

    // --- String ---

    /// Builtin term for [`DefaultFunction::AppendString`].
    pub fn append_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::AppendString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::EqualsString`].
    pub fn equals_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::EqualsString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::EncodeUtf8`].
    pub fn encode_utf8(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::EncodeUtf8);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::DecodeUtf8`].
    pub fn decode_utf8(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::DecodeUtf8);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ChooseUnit`].
    pub fn choose_unit(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ChooseUnit);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Trace`].
    pub fn trace(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Trace);

        Term::builtin(arena, fun)
    }

    // --- Pairs ---

    /// Builtin term for [`DefaultFunction::FstPair`].
    pub fn fst_pair(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::FstPair);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::SndPair`].
    pub fn snd_pair(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::SndPair);

        Term::builtin(arena, fun)
    }

    // --- Lists ---

    /// Builtin term for [`DefaultFunction::ChooseList`].
    pub fn choose_list(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ChooseList);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MkCons`].
    pub fn mk_cons(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MkCons);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::HeadList`].
    pub fn head_list(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::HeadList);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::TailList`].
    pub fn tail_list(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::TailList);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::NullList`].
    pub fn null_list(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::NullList);

        Term::builtin(arena, fun)
    }

    // --- Data ---

    /// Builtin term for [`DefaultFunction::ChooseData`].
    pub fn choose_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ChooseData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ConstrData`].
    pub fn constr_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ConstrData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MapData`].
    pub fn map_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MapData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ListData`].
    pub fn list_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ListData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::IData`].
    pub fn i_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::IData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::BData`].
    pub fn b_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::BData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::UnConstrData`].
    pub fn un_constr_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnConstrData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::UnMapData`].
    pub fn un_map_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnMapData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::UnListData`].
    pub fn un_list_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnListData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::UnIData`].
    pub fn un_i_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnIData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::UnBData`].
    pub fn un_b_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnBData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::EqualsData`].
    pub fn equals_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::EqualsData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MkPairData`].
    pub fn mk_pair_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MkPairData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MkNilData`].
    pub fn mk_nil_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MkNilData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::MkNilPairData`].
    pub fn mk_nil_pair_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::MkNilPairData);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::SerialiseData`].
    pub fn serialise_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::SerialiseData);

        Term::builtin(arena, fun)
    }

    // --- BLS12-381 ---

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_Add`].
    pub fn bls12_381_g1_add(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_Add);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_Neg`].
    pub fn bls12_381_g1_neg(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_Neg);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_ScalarMul`].
    pub fn bls12_381_g1_scalar_mul(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_ScalarMul);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_Equal`].
    pub fn bls12_381_g1_equal(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_Equal);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_Compress`].
    pub fn bls12_381_g1_compress(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_Compress);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_Uncompress`].
    pub fn bls12_381_g1_uncompress(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_Uncompress);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G1_HashToGroup`].
    pub fn bls12_381_g1_hash_to_group(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_HashToGroup);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_Add`].
    pub fn bls12_381_g2_add(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_Add);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_Neg`].
    pub fn bls12_381_g2_neg(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_Neg);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_ScalarMul`].
    pub fn bls12_381_g2_scalar_mul(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_ScalarMul);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_Equal`].
    pub fn bls12_381_g2_equal(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_Equal);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_Compress`].
    pub fn bls12_381_g2_compress(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_Compress);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_Uncompress`].
    pub fn bls12_381_g2_uncompress(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_Uncompress);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_G2_HashToGroup`].
    pub fn bls12_381_g2_hash_to_group(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_HashToGroup);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_MillerLoop`].
    pub fn bls12_381_miller_loop(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_MillerLoop);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_MulMlResult`].
    pub fn bls12_381_mul_ml_result(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_MulMlResult);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Bls12_381_FinalVerify`].
    pub fn bls12_381_final_verify(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_FinalVerify);

        Term::builtin(arena, fun)
    }

    // --- Bitwise ---

    /// Builtin term for [`DefaultFunction::IntegerToByteString`].
    pub fn integer_to_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::IntegerToByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ByteStringToInteger`].
    pub fn byte_string_to_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ByteStringToInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::AndByteString`].
    pub fn and_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::AndByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::OrByteString`].
    pub fn or_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::OrByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::XorByteString`].
    pub fn xor_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::XorByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ComplementByteString`].
    pub fn complement_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ComplementByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ReadBit`].
    pub fn read_bit(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ReadBit);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::WriteBits`].
    pub fn write_bits(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::WriteBits);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ReplicateByte`].
    pub fn replicate_byte(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ReplicateByte);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ShiftByteString`].
    pub fn shift_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ShiftByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::RotateByteString`].
    pub fn rotate_byte_string(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::RotateByteString);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::CountSetBits`].
    pub fn count_set_bits(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::CountSetBits);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::FindFirstSetBit`].
    pub fn find_first_set_bit(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::FindFirstSetBit);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::Ripemd_160`].
    pub fn ripemd_160(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Ripemd_160);

        Term::builtin(arena, fun)
    }

    // --- van Rossem builtins ---

    /// Builtin term for [`DefaultFunction::ExpModInteger`].
    pub fn exp_mod_integer(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ExpModInteger);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::DropList`].
    pub fn drop_list(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::DropList);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::LengthOfArray`].
    pub fn length_of_array(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LengthOfArray);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::ListToArray`].
    pub fn list_to_array(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ListToArray);

        Term::builtin(arena, fun)
    }

    /// Builtin term for [`DefaultFunction::IndexArray`].
    pub fn index_array(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::IndexArray);

        Term::builtin(arena, fun)
    }

    pub fn bls12_381_g1_multi_scalar_mul(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G1_MultiScalarMul);

        Term::builtin(arena, fun)
    }

    pub fn bls12_381_g2_multi_scalar_mul(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::Bls12_381_G2_MultiScalarMul);

        Term::builtin(arena, fun)
    }

    pub fn insert_coin(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::InsertCoin);

        Term::builtin(arena, fun)
    }

    pub fn lookup_coin(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::LookupCoin);

        Term::builtin(arena, fun)
    }

    pub fn union_value(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnionValue);

        Term::builtin(arena, fun)
    }

    pub fn value_contains(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ValueContains);

        Term::builtin(arena, fun)
    }

    pub fn value_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ValueData);

        Term::builtin(arena, fun)
    }

    pub fn un_value_data(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::UnValueData);

        Term::builtin(arena, fun)
    }

    pub fn scale_value(arena: &'a Arena) -> &'a Term<'a, V> {
        let fun = arena.alloc(DefaultFunction::ScaleValue);

        Term::builtin(arena, fun)
    }
}
