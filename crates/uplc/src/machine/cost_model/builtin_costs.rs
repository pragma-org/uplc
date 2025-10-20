use crate::machine::{cost_model::cost_map::CostMap, ExBudget, PlutusVersion};

use super::costing::{
    Cost, OneArgumentCosting, SixArgumentsCosting, ThreeArgumentsCosting, TwoArgumentsCosting,
};

#[derive(Debug, PartialEq)]
pub struct BuiltinCosts {
    add_integer: TwoArgumentsCosting,
    subtract_integer: TwoArgumentsCosting,
    multiply_integer: TwoArgumentsCosting,
    divide_integer: TwoArgumentsCosting,
    quotient_integer: TwoArgumentsCosting,
    remainder_integer: TwoArgumentsCosting,
    mod_integer: TwoArgumentsCosting,
    equals_integer: TwoArgumentsCosting,
    less_than_integer: TwoArgumentsCosting,
    less_than_equals_integer: TwoArgumentsCosting,
    // Bytestrings
    append_byte_string: TwoArgumentsCosting,
    cons_byte_string: TwoArgumentsCosting,
    slice_byte_string: ThreeArgumentsCosting,
    length_of_byte_string: OneArgumentCosting,
    index_byte_string: TwoArgumentsCosting,
    equals_byte_string: TwoArgumentsCosting,
    less_than_byte_string: TwoArgumentsCosting,
    less_than_equals_byte_string: TwoArgumentsCosting,
    // Cryptography and hashes
    sha2_256: OneArgumentCosting,
    sha3_256: OneArgumentCosting,
    blake2b_224: OneArgumentCosting,
    blake2b_256: OneArgumentCosting,
    keccak_256: OneArgumentCosting,
    verify_ed25519_signature: ThreeArgumentsCosting,
    verify_ecdsa_secp256k1_signature: ThreeArgumentsCosting,
    verify_schnorr_secp256k1_signature: ThreeArgumentsCosting,
    // Strings
    append_string: TwoArgumentsCosting,
    equals_string: TwoArgumentsCosting,
    encode_utf8: OneArgumentCosting,
    decode_utf8: OneArgumentCosting,
    // Bool
    if_then_else: ThreeArgumentsCosting,
    // Unit
    choose_unit: TwoArgumentsCosting,
    // Tracing
    trace: TwoArgumentsCosting,
    // Pairs
    fst_pair: OneArgumentCosting,
    snd_pair: OneArgumentCosting,
    // Lists
    choose_list: ThreeArgumentsCosting,
    mk_cons: TwoArgumentsCosting,
    head_list: OneArgumentCosting,
    tail_list: OneArgumentCosting,
    null_list: OneArgumentCosting,
    // Data
    choose_data: SixArgumentsCosting,
    constr_data: TwoArgumentsCosting,
    map_data: OneArgumentCosting,
    list_data: OneArgumentCosting,
    i_data: OneArgumentCosting,
    b_data: OneArgumentCosting,
    un_constr_data: OneArgumentCosting,
    un_map_data: OneArgumentCosting,
    un_list_data: OneArgumentCosting,
    un_i_data: OneArgumentCosting,
    un_b_data: OneArgumentCosting,
    equals_data: TwoArgumentsCosting,
    // Misc constructors
    mk_pair_data: TwoArgumentsCosting,
    mk_nil_data: OneArgumentCosting,
    mk_nil_pair_data: OneArgumentCosting,
    serialise_data: OneArgumentCosting,
    // BLST
    bls12_381_g1_add: TwoArgumentsCosting,
    bls12_381_g1_neg: OneArgumentCosting,
    bls12_381_g1_scalar_mul: TwoArgumentsCosting,
    bls12_381_g1_equal: TwoArgumentsCosting,
    bls12_381_g1_compress: OneArgumentCosting,
    bls12_381_g1_uncompress: OneArgumentCosting,
    bls12_381_g1_hash_to_group: TwoArgumentsCosting,
    bls12_381_g2_add: TwoArgumentsCosting,
    bls12_381_g2_neg: OneArgumentCosting,
    bls12_381_g2_scalar_mul: TwoArgumentsCosting,
    bls12_381_g2_equal: TwoArgumentsCosting,
    bls12_381_g2_compress: OneArgumentCosting,
    bls12_381_g2_uncompress: OneArgumentCosting,
    bls12_381_g2_hash_to_group: TwoArgumentsCosting,
    bls12_381_miller_loop: TwoArgumentsCosting,
    bls12_381_mul_ml_result: TwoArgumentsCosting,
    bls12_381_final_verify: TwoArgumentsCosting,
    // bitwise
    integer_to_byte_string: ThreeArgumentsCosting,
    byte_string_to_integer: TwoArgumentsCosting,
    and_byte_string: ThreeArgumentsCosting,
    or_byte_string: ThreeArgumentsCosting,
    xor_byte_string: ThreeArgumentsCosting,
    complement_byte_string: OneArgumentCosting,
    read_bit: TwoArgumentsCosting,
    write_bits: ThreeArgumentsCosting,
    replicate_byte: TwoArgumentsCosting,
    shift_byte_string: TwoArgumentsCosting,
    rotate_byte_string: TwoArgumentsCosting,
    count_set_bits: OneArgumentCosting,
    find_first_set_bit: OneArgumentCosting,
    ripemd_160: OneArgumentCosting,

    exp_mod_integer: ThreeArgumentsCosting,
    drop_list: TwoArgumentsCosting,
    length_of_array: OneArgumentCosting,
    list_to_array: TwoArgumentsCosting,
    index_array: TwoArgumentsCosting,
}

impl Default for BuiltinCosts {
    fn default() -> Self {
        BuiltinCosts::v3()
    }
}

impl BuiltinCosts {
    pub fn add_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.add_integer.mem.cost(args),
            self.add_integer.cpu.cost(args),
        )
    }

    pub fn subtract_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.subtract_integer.mem.cost(args),
            self.subtract_integer.cpu.cost(args),
        )
    }

    pub fn equals_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.equals_integer.mem.cost(args),
            self.equals_integer.cpu.cost(args),
        )
    }

    pub fn less_than_equals_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.less_than_equals_integer.mem.cost(args),
            self.less_than_equals_integer.cpu.cost(args),
        )
    }

    pub fn multiply_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.multiply_integer.mem.cost(args),
            self.multiply_integer.cpu.cost(args),
        )
    }

    pub fn divide_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.divide_integer.mem.cost(args),
            self.divide_integer.cpu.cost(args),
        )
    }

    pub fn quotient_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.quotient_integer.mem.cost(args),
            self.quotient_integer.cpu.cost(args),
        )
    }

    pub fn remainder_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.remainder_integer.mem.cost(args),
            self.remainder_integer.cpu.cost(args),
        )
    }

    pub fn mod_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.mod_integer.mem.cost(args),
            self.mod_integer.cpu.cost(args),
        )
    }

    pub fn less_than_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.less_than_integer.mem.cost(args),
            self.less_than_integer.cpu.cost(args),
        )
    }

    pub fn append_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.append_byte_string.mem.cost(args),
            self.append_byte_string.cpu.cost(args),
        )
    }

    pub fn equals_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.equals_byte_string.mem.cost(args),
            self.equals_byte_string.cpu.cost(args),
        )
    }

    pub fn cons_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.cons_byte_string.mem.cost(args),
            self.cons_byte_string.cpu.cost(args),
        )
    }

    pub fn slice_byte_string(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.slice_byte_string.mem.cost(args),
            self.slice_byte_string.cpu.cost(args),
        )
    }

    pub fn length_of_byte_string(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.length_of_byte_string.mem.cost(args),
            self.length_of_byte_string.cpu.cost(args),
        )
    }

    pub fn index_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.index_byte_string.mem.cost(args),
            self.index_byte_string.cpu.cost(args),
        )
    }

    pub fn less_than_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.less_than_byte_string.mem.cost(args),
            self.less_than_byte_string.cpu.cost(args),
        )
    }

    pub fn less_than_equals_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.less_than_equals_byte_string.mem.cost(args),
            self.less_than_equals_byte_string.cpu.cost(args),
        )
    }

    pub fn sha2_256(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.sha2_256.mem.cost(args), self.sha2_256.cpu.cost(args))
    }

    pub fn sha3_256(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.sha3_256.mem.cost(args), self.sha3_256.cpu.cost(args))
    }

    pub fn blake2b_256(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.blake2b_256.mem.cost(args),
            self.blake2b_256.cpu.cost(args),
        )
    }

    pub fn keccak_256(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.keccak_256.mem.cost(args),
            self.keccak_256.cpu.cost(args),
        )
    }

    pub fn blake2b_224(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.blake2b_224.mem.cost(args),
            self.blake2b_224.cpu.cost(args),
        )
    }

    pub fn verify_ed25519_signature(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.verify_ed25519_signature.mem.cost(args),
            self.verify_ed25519_signature.cpu.cost(args),
        )
    }

    pub fn verify_ecdsa_secp256k1_signature(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.verify_ecdsa_secp256k1_signature.mem.cost(args),
            self.verify_ecdsa_secp256k1_signature.cpu.cost(args),
        )
    }

    pub fn verify_schnorr_secp256k1_signature(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.verify_schnorr_secp256k1_signature.mem.cost(args),
            self.verify_schnorr_secp256k1_signature.cpu.cost(args),
        )
    }

    pub fn append_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.append_string.mem.cost(args),
            self.append_string.cpu.cost(args),
        )
    }

    pub fn equals_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.equals_string.mem.cost(args),
            self.equals_string.cpu.cost(args),
        )
    }

    pub fn encode_utf8(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.encode_utf8.mem.cost(args),
            self.encode_utf8.cpu.cost(args),
        )
    }

    pub fn decode_utf8(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.decode_utf8.mem.cost(args),
            self.decode_utf8.cpu.cost(args),
        )
    }

    pub fn if_then_else(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.if_then_else.mem.cost(args),
            self.if_then_else.cpu.cost(args),
        )
    }

    pub fn choose_unit(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.choose_unit.mem.cost(args),
            self.choose_unit.cpu.cost(args),
        )
    }

    pub fn trace(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(self.trace.mem.cost(args), self.trace.cpu.cost(args))
    }

    pub fn fst_pair(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.fst_pair.mem.cost(args), self.fst_pair.cpu.cost(args))
    }

    pub fn snd_pair(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.snd_pair.mem.cost(args), self.snd_pair.cpu.cost(args))
    }

    pub fn choose_list(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.choose_list.mem.cost(args),
            self.choose_list.cpu.cost(args),
        )
    }

    pub fn mk_cons(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(self.mk_cons.mem.cost(args), self.mk_cons.cpu.cost(args))
    }

    pub fn head_list(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.head_list.mem.cost(args), self.head_list.cpu.cost(args))
    }

    pub fn tail_list(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.tail_list.mem.cost(args), self.tail_list.cpu.cost(args))
    }

    pub fn null_list(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.null_list.mem.cost(args), self.null_list.cpu.cost(args))
    }

    pub fn choose_data(&self, args: [i64; 6]) -> ExBudget {
        ExBudget::new(
            self.choose_data.mem.cost(args),
            self.choose_data.cpu.cost(args),
        )
    }

    pub fn constr_data(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.constr_data.mem.cost(args),
            self.constr_data.cpu.cost(args),
        )
    }

    pub fn map_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.map_data.mem.cost(args), self.map_data.cpu.cost(args))
    }

    pub fn list_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.list_data.mem.cost(args), self.list_data.cpu.cost(args))
    }

    pub fn i_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.i_data.mem.cost(args), self.i_data.cpu.cost(args))
    }

    pub fn b_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.b_data.mem.cost(args), self.b_data.cpu.cost(args))
    }

    pub fn un_constr_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.un_constr_data.mem.cost(args),
            self.un_constr_data.cpu.cost(args),
        )
    }

    pub fn un_map_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.un_map_data.mem.cost(args),
            self.un_map_data.cpu.cost(args),
        )
    }

    pub fn un_list_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.un_list_data.mem.cost(args),
            self.un_list_data.cpu.cost(args),
        )
    }

    pub fn un_i_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.un_i_data.mem.cost(args), self.un_i_data.cpu.cost(args))
    }

    pub fn un_b_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(self.un_b_data.mem.cost(args), self.un_b_data.cpu.cost(args))
    }

    pub fn equals_data(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.equals_data.mem.cost(args),
            self.equals_data.cpu.cost(args),
        )
    }

    pub fn mk_pair_data(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.mk_pair_data.mem.cost(args),
            self.mk_pair_data.cpu.cost(args),
        )
    }

    pub fn mk_nil_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.mk_nil_data.mem.cost(args),
            self.mk_nil_data.cpu.cost(args),
        )
    }

    pub fn mk_nil_pair_data(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.mk_nil_pair_data.mem.cost(args),
            self.mk_nil_pair_data.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_add(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_add.mem.cost(args),
            self.bls12_381_g1_add.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_neg(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_neg.mem.cost(args),
            self.bls12_381_g1_neg.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_scalar_mul(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_scalar_mul.mem.cost(args),
            self.bls12_381_g1_scalar_mul.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_equal(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_equal.mem.cost(args),
            self.bls12_381_g1_equal.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_compress(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_compress.mem.cost(args),
            self.bls12_381_g1_compress.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_uncompress(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_uncompress.mem.cost(args),
            self.bls12_381_g1_uncompress.cpu.cost(args),
        )
    }

    pub fn bls12_381_g1_hash_to_group(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g1_hash_to_group.mem.cost(args),
            self.bls12_381_g1_hash_to_group.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_add(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_add.mem.cost(args),
            self.bls12_381_g2_add.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_neg(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_neg.mem.cost(args),
            self.bls12_381_g2_neg.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_scalar_mul(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_scalar_mul.mem.cost(args),
            self.bls12_381_g2_scalar_mul.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_equal(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_equal.mem.cost(args),
            self.bls12_381_g2_equal.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_compress(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_compress.mem.cost(args),
            self.bls12_381_g2_compress.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_uncompress(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_uncompress.mem.cost(args),
            self.bls12_381_g2_uncompress.cpu.cost(args),
        )
    }

    pub fn bls12_381_g2_hash_to_group(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_g2_hash_to_group.mem.cost(args),
            self.bls12_381_g2_hash_to_group.cpu.cost(args),
        )
    }

    pub fn bls12_381_miller_loop(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_miller_loop.mem.cost(args),
            self.bls12_381_miller_loop.cpu.cost(args),
        )
    }

    pub fn bls12_381_mul_ml_result(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_mul_ml_result.mem.cost(args),
            self.bls12_381_mul_ml_result.cpu.cost(args),
        )
    }

    pub fn bls12_381_final_verify(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.bls12_381_final_verify.mem.cost(args),
            self.bls12_381_final_verify.cpu.cost(args),
        )
    }

    pub fn integer_to_byte_string(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.integer_to_byte_string.mem.cost(args),
            self.integer_to_byte_string.cpu.cost(args),
        )
    }

    pub fn byte_string_to_integer(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.byte_string_to_integer.mem.cost(args),
            self.byte_string_to_integer.cpu.cost(args),
        )
    }

    pub fn and_byte_string(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.and_byte_string.mem.cost(args),
            self.and_byte_string.cpu.cost(args),
        )
    }

    pub fn or_byte_string(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.or_byte_string.mem.cost(args),
            self.or_byte_string.cpu.cost(args),
        )
    }

    pub fn xor_byte_string(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.xor_byte_string.mem.cost(args),
            self.xor_byte_string.cpu.cost(args),
        )
    }

    pub fn complement_byte_string(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.complement_byte_string.mem.cost(args),
            self.complement_byte_string.cpu.cost(args),
        )
    }

    pub fn read_bit(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(self.read_bit.mem.cost(args), self.read_bit.cpu.cost(args))
    }

    pub fn write_bits(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.write_bits.mem.cost(args),
            self.write_bits.cpu.cost(args),
        )
    }

    pub fn replicate_byte(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.replicate_byte.mem.cost(args),
            self.replicate_byte.cpu.cost(args),
        )
    }

    pub fn shift_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.shift_byte_string.mem.cost(args),
            self.shift_byte_string.cpu.cost(args),
        )
    }

    pub fn rotate_byte_string(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.rotate_byte_string.mem.cost(args),
            self.rotate_byte_string.cpu.cost(args),
        )
    }

    pub fn count_set_bits(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.count_set_bits.mem.cost(args),
            self.count_set_bits.cpu.cost(args),
        )
    }

    pub fn find_first_set_bit(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.find_first_set_bit.mem.cost(args),
            self.find_first_set_bit.cpu.cost(args),
        )
    }

    pub fn ripemd_160(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.ripemd_160.mem.cost(args),
            self.ripemd_160.cpu.cost(args),
        )
    }

    pub fn exp_mod_integer(&self, args: [i64; 3]) -> ExBudget {
        ExBudget::new(
            self.exp_mod_integer.mem.cost(args),
            self.exp_mod_integer.cpu.cost(args),
        )
    }

    pub fn drop_list(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(self.drop_list.mem.cost(args), self.drop_list.cpu.cost(args))
    }

    pub fn length_of_array(&self, args: [i64; 1]) -> ExBudget {
        ExBudget::new(
            self.length_of_array.mem.cost(args),
            self.length_of_array.cpu.cost(args),
        )
    }

    pub fn list_to_array(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.list_to_array.mem.cost(args),
            self.list_to_array.cpu.cost(args),
        )
    }

    pub fn index_array(&self, args: [i64; 2]) -> ExBudget {
        ExBudget::new(
            self.index_array.mem.cost(args),
            self.index_array.cpu.cost(args),
        )
    }

    pub fn v1() -> Self {
        Self {
            add_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),
            subtract_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),
            multiply_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::multiplied_sizes(90434, 519),
            ),
            divide_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            quotient_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            remainder_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            mod_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(51775, 558),
            ),
            less_than_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(44749, 541),
            ),
            less_than_equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(43285, 552),
            ),
            append_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::added_sizes(1000, 173),
            ),
            cons_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::linear_in_y(72010, 178),
            ),
            slice_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_z(4, 0),
                ThreeArgumentsCosting::linear_in_z(20467, 1),
            ),
            length_of_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(22100),
            ),
            index_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(13169),
            ),
            equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(24548, 29498, 38),
            ),
            less_than_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            less_than_equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            sha2_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(270652, 22588),
            ),
            sha3_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(1457325, 64566),
            ),
            blake2b_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(201305, 8356),
            ),
            verify_ed25519_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::linear_in_y(53384111, 14333),
            ),
            verify_ecdsa_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            verify_schnorr_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            append_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(4, 1),
                TwoArgumentsCosting::added_sizes(1000, 59957),
            ),
            equals_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(39184, 1000, 60594),
            ),
            encode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(1000, 42921),
            ),
            decode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(91189, 769),
            ),
            if_then_else: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(1),
                ThreeArgumentsCosting::constant_cost(76049),
            ),
            choose_unit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(61462),
            ),
            trace: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(59498),
            ),
            fst_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141895),
            ),
            snd_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141992),
            ),
            choose_list: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(32),
                ThreeArgumentsCosting::constant_cost(132994),
            ),
            mk_cons: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(72362),
            ),
            head_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(83150),
            ),
            tail_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(81663),
            ),
            null_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(74433),
            ),
            choose_data: SixArgumentsCosting::new(
                SixArgumentsCosting::constant_cost(32),
                SixArgumentsCosting::constant_cost(94375),
            ),
            constr_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(22151),
            ),
            map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(68246),
            ),
            list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(33852),
            ),
            i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(15299),
            ),
            b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(11183),
            ),
            un_constr_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24588),
            ),
            un_map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24623),
            ),
            un_list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(25933),
            ),
            un_i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20744),
            ),
            un_b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20142),
            ),
            equals_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(898148, 27279),
            ),
            mk_pair_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(11546),
            ),
            mk_nil_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7243),
            ),
            mk_nil_pair_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7391),
            ),
            serialise_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            blake2b_224: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            keccak_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_miller_loop: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_mul_ml_result: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_final_verify: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            integer_to_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            byte_string_to_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            and_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            or_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            xor_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            complement_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            read_bit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            write_bits: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            replicate_byte: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            shift_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            rotate_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            count_set_bits: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            find_first_set_bit: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            ripemd_160: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            exp_mod_integer: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            drop_list: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::linear_in_x(116711, 1957),
            ),
            length_of_array: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(198994),
            ),
            list_to_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(7, 1),
                TwoArgumentsCosting::linear_in_x(307802, 8496),
            ),
            index_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(194922),
            ),
        }
    }

    pub fn v2() -> Self {
        Self {
            add_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),
            subtract_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),
            multiply_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::multiplied_sizes(90434, 519),
            ),
            divide_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            quotient_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            remainder_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            mod_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(85848, 228465, 122),
            ),
            equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(51775, 558),
            ),
            less_than_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(44749, 541),
            ),
            less_than_equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(43285, 552),
            ),
            append_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::added_sizes(1000, 173),
            ),
            cons_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::linear_in_y(72010, 178),
            ),
            slice_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_z(4, 0),
                ThreeArgumentsCosting::linear_in_z(20467, 1),
            ),
            length_of_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(22100),
            ),
            index_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(13169),
            ),
            equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(24548, 29498, 38),
            ),
            less_than_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            less_than_equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            sha2_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(270652, 22588),
            ),
            sha3_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(1457325, 64566),
            ),
            blake2b_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(201305, 8356),
            ),
            verify_ed25519_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::linear_in_y(53384111, 14333),
            ),
            verify_ecdsa_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::constant_cost(43053543),
            ),
            verify_schnorr_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::linear_in_y(43574283, 26308),
            ),
            append_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(4, 1),
                TwoArgumentsCosting::added_sizes(1000, 59957),
            ),
            equals_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(39184, 1000, 60594),
            ),
            encode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(1000, 42921),
            ),
            decode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(91189, 769),
            ),
            if_then_else: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(1),
                ThreeArgumentsCosting::constant_cost(76049),
            ),
            choose_unit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(61462),
            ),
            trace: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(59498),
            ),
            fst_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141895),
            ),
            snd_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141992),
            ),
            choose_list: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(32),
                ThreeArgumentsCosting::constant_cost(132994),
            ),
            mk_cons: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(72362),
            ),
            head_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(83150),
            ),
            tail_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(81663),
            ),
            null_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(74433),
            ),
            choose_data: SixArgumentsCosting::new(
                SixArgumentsCosting::constant_cost(32),
                SixArgumentsCosting::constant_cost(94375),
            ),
            constr_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(22151),
            ),
            map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(68246),
            ),
            list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(33852),
            ),
            i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(15299),
            ),
            b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(11183),
            ),
            un_constr_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24588),
            ),
            un_map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24623),
            ),
            un_list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(25933),
            ),
            un_i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20744),
            ),
            un_b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20142),
            ),
            equals_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(898148, 27279),
            ),
            mk_pair_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(11546),
            ),
            mk_nil_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7243),
            ),
            mk_nil_pair_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7391),
            ),
            serialise_data: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(0, 2),
                OneArgumentCosting::linear_cost(955506, 213312),
            ),
            blake2b_224: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            keccak_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g1_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            bls12_381_g2_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_miller_loop: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_mul_ml_result: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            bls12_381_final_verify: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            integer_to_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            byte_string_to_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            and_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            or_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            xor_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            complement_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            read_bit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            write_bits: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            replicate_byte: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            shift_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            rotate_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(30000000000),
                TwoArgumentsCosting::constant_cost(30000000000),
            ),
            count_set_bits: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            find_first_set_bit: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            ripemd_160: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(30000000000),
                OneArgumentCosting::constant_cost(30000000000),
            ),
            exp_mod_integer: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(30000000000),
                ThreeArgumentsCosting::constant_cost(30000000000),
            ),
            drop_list: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::linear_in_x(116711, 1957),
            ),
            length_of_array: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(198994),
            ),
            list_to_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(7, 1),
                TwoArgumentsCosting::linear_in_x(307802, 8496),
            ),
            index_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(194922),
            ),
        }
    }

    pub fn v3() -> Self {
        Self {
            add_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),
            subtract_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(1, 1),
                TwoArgumentsCosting::max_size(100788, 420),
            ),

            multiply_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::multiplied_sizes(90434, 519),
            ),
            divide_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                    85848, 85848, 123203, 7305, -900, 1716, 549, 57,
                ),
            ),
            quotient_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(0, 1, 1),
                TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                    85848, 85848, 123203, 7305, -900, 1716, 549, 57,
                ),
            ),
            remainder_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_y(0, 1),
                TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                    85848, 85848, 123203, 7305, -900, 1716, 549, 57,
                ),
            ),
            mod_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_y(0, 1),
                TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                    85848, 85848, 123203, 7305, -900, 1716, 549, 57,
                ),
            ),
            equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(51775, 558),
            ),
            less_than_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(44749, 541),
            ),
            less_than_equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(43285, 552),
            ),
            append_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::added_sizes(1000, 173),
            ),
            cons_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(0, 1),
                TwoArgumentsCosting::linear_in_y(72010, 178),
            ),
            slice_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_z(4, 0),
                ThreeArgumentsCosting::linear_in_z(20467, 1),
            ),
            length_of_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(22100),
            ),
            index_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(13169),
            ),
            equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(24548, 29498, 38),
            ),
            less_than_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            less_than_equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(28999, 74),
            ),
            sha2_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(270652, 22588),
            ),
            sha3_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(1457325, 64566),
            ),
            blake2b_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(201305, 8356),
            ),
            verify_ed25519_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::linear_in_y(53384111, 14333),
            ),
            verify_ecdsa_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::constant_cost(43053543),
            ),
            verify_schnorr_secp256k1_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(10),
                ThreeArgumentsCosting::linear_in_y(43574283, 26308),
            ),
            append_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(4, 1),
                TwoArgumentsCosting::added_sizes(1000, 59957),
            ),
            equals_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::linear_on_diagonal(39184, 1000, 60594),
            ),
            encode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(1000, 42921),
            ),
            decode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(4, 2),
                OneArgumentCosting::linear_cost(91189, 769),
            ),
            if_then_else: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(1),
                ThreeArgumentsCosting::constant_cost(76049),
            ),
            choose_unit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::constant_cost(61462),
            ),
            trace: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(59498),
            ),
            fst_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141895),
            ),
            snd_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(141992),
            ),
            choose_list: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(32),
                ThreeArgumentsCosting::constant_cost(132994),
            ),
            mk_cons: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(72362),
            ),
            head_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(83150),
            ),
            tail_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(81663),
            ),
            null_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(74433),
            ),
            choose_data: SixArgumentsCosting::new(
                SixArgumentsCosting::constant_cost(32),
                SixArgumentsCosting::constant_cost(94375),
            ),
            constr_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(22151),
            ),
            map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(68246),
            ),
            list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(33852),
            ),
            i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(15299),
            ),
            b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(11183),
            ),
            un_constr_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24588),
            ),
            un_map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(24623),
            ),
            un_list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(25933),
            ),
            un_i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20744),
            ),
            un_b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(20142),
            ),
            equals_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::min_size(898148, 27279),
            ),
            mk_pair_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(11546),
            ),
            mk_nil_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7243),
            ),
            mk_nil_pair_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(32),
                OneArgumentCosting::constant_cost(7391),
            ),

            serialise_data: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(0, 2),
                OneArgumentCosting::linear_cost(955506, 213312),
            ),
            blake2b_224: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(207616, 8310),
            ),
            keccak_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(4),
                OneArgumentCosting::linear_cost(2261318, 64571),
            ),
            bls12_381_g1_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(18),
                TwoArgumentsCosting::constant_cost(962335),
            ),
            bls12_381_g1_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(18),
                OneArgumentCosting::constant_cost(267929),
            ),
            bls12_381_g1_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(18),
                TwoArgumentsCosting::linear_in_x(76433006, 8868),
            ),
            bls12_381_g1_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::constant_cost(442008),
            ),
            bls12_381_g1_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(6),
                OneArgumentCosting::constant_cost(2780678),
            ),
            bls12_381_g1_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(18),
                OneArgumentCosting::constant_cost(52948122),
            ),
            bls12_381_g1_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(18),
                TwoArgumentsCosting::linear_in_x(52538055, 3756),
            ),
            bls12_381_g2_add: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(36),
                TwoArgumentsCosting::constant_cost(1995836),
            ),
            bls12_381_g2_neg: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(36),
                OneArgumentCosting::constant_cost(284546),
            ),
            bls12_381_g2_scalar_mul: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(36),
                TwoArgumentsCosting::linear_in_x(158_221_314, 26_549),
            ),
            bls12_381_g2_equal: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::constant_cost(901_022),
            ),
            bls12_381_g2_compress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(12),
                OneArgumentCosting::constant_cost(3_227_919),
            ),
            bls12_381_g2_uncompress: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(36),
                OneArgumentCosting::constant_cost(74_698_472),
            ),
            bls12_381_g2_hash_to_group: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(36),
                TwoArgumentsCosting::linear_in_x(166_917_843, 4_307),
            ),
            bls12_381_miller_loop: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(72),
                TwoArgumentsCosting::constant_cost(254006273),
            ),
            bls12_381_mul_ml_result: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(72),
                TwoArgumentsCosting::constant_cost(2174038),
            ),
            bls12_381_final_verify: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::constant_cost(333849714),
            ),
            integer_to_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::literal_in_y_or_linear_in_z(0, 1),
                ThreeArgumentsCosting::quadratic_in_z(1293828, 28716, 63),
            ),
            byte_string_to_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_y(0, 1),
                TwoArgumentsCosting::quadratic_in_y(1006041, 43623, 251),
            ),
            and_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_max_y_z(0, 1),
                ThreeArgumentsCosting::linear_in_y_and_z(100181, 726, 719),
            ),
            or_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_max_y_z(0, 1),
                ThreeArgumentsCosting::linear_in_y_and_z(100181, 726, 719),
            ),
            xor_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_max_y_z(0, 1),
                ThreeArgumentsCosting::linear_in_y_and_z(100181, 726, 719),
            ),
            complement_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(0, 1),
                OneArgumentCosting::linear_cost(107878, 680),
            ),
            read_bit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(1),
                TwoArgumentsCosting::constant_cost(95336),
            ),
            write_bits: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_x(0, 1),
                ThreeArgumentsCosting::linear_in_y(281145, 18848),
            ),
            replicate_byte: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(1, 1),
                TwoArgumentsCosting::linear_in_x(180194, 159),
            ),
            shift_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(0, 1),
                TwoArgumentsCosting::linear_in_x(158519, 8942),
            ),
            rotate_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(0, 1),
                TwoArgumentsCosting::linear_in_x(159378, 8813),
            ),
            count_set_bits: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(1),
                OneArgumentCosting::linear_cost(107490, 3298),
            ),
            find_first_set_bit: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(1),
                OneArgumentCosting::linear_cost(106057, 655),
            ),
            ripemd_160: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(3),
                OneArgumentCosting::linear_cost(1964219, 24520),
            ),
            exp_mod_integer: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_z(0, 1),
                ThreeArgumentsCosting::exp_mod_cost(607153, 231697, 53144),
            ),
            drop_list: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::linear_in_x(116711, 1957),
            ),
            length_of_array: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(198994),
            ),
            list_to_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(7, 1),
                TwoArgumentsCosting::linear_in_x(307802, 8496),
            ),
            index_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(194922),
            ),
        }
    }

    pub fn initialize_builtin_costs(version: &PlutusVersion, cost_map: &CostMap) -> Self {
        Self {
            add_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(
                    cost_map["add_integer-mem-arguments-intercept"],
                    cost_map["add_integer-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::max_size(
                    cost_map["add_integer-cpu-arguments-intercept"],
                    cost_map["add_integer-cpu-arguments-slope"],
                ),
            ),

            append_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(
                    cost_map["append_byte_string-mem-arguments-intercept"],
                    cost_map["append_byte_string-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::added_sizes(
                    cost_map["append_byte_string-cpu-arguments-intercept"],
                    cost_map["append_byte_string-cpu-arguments-slope"],
                ),
            ),

            append_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(
                    cost_map["append_string-mem-arguments-intercept"],
                    cost_map["append_string-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::added_sizes(
                    cost_map["append_string-cpu-arguments-intercept"],
                    cost_map["append_string-cpu-arguments-slope"],
                ),
            ),

            b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["b_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["b_data-cpu-arguments"]),
            ),

            blake2b_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["blake2b_256-mem-arguments"]),
                OneArgumentCosting::linear_cost(
                    cost_map["blake2b_256-cpu-arguments-intercept"],
                    cost_map["blake2b_256-cpu-arguments-slope"],
                ),
            ),
            choose_data: SixArgumentsCosting::new(
                SixArgumentsCosting::constant_cost(cost_map["choose_data-mem-arguments"]),
                SixArgumentsCosting::constant_cost(cost_map["choose_data-cpu-arguments"]),
            ),
            choose_list: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(cost_map["choose_list-mem-arguments"]),
                ThreeArgumentsCosting::constant_cost(cost_map["choose_list-cpu-arguments"]),
            ),
            choose_unit: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["choose_unit-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["choose_unit-cpu-arguments"]),
            ),
            cons_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(
                    cost_map["cons_byte_string-mem-arguments-intercept"],
                    cost_map["cons_byte_string-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::linear_in_y(
                    cost_map["cons_byte_string-cpu-arguments-intercept"],
                    cost_map["cons_byte_string-cpu-arguments-slope"],
                ),
            ),
            constr_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["constr_data-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["constr_data-cpu-arguments"]),
            ),
            decode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(
                    cost_map["decode_utf8-mem-arguments-intercept"],
                    cost_map["decode_utf8-mem-arguments-slope"],
                ),
                OneArgumentCosting::linear_cost(
                    cost_map["decode_utf8-cpu-arguments-intercept"],
                    cost_map["decode_utf8-cpu-arguments-slope"],
                ),
            ),
            divide_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(
                    cost_map["divide_integer-mem-arguments-intercept"],
                    cost_map["divide_integer-mem-arguments-minimum"],
                    cost_map["divide_integer-mem-arguments-slope"],
                ),
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => {
                        TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(
                            cost_map["divide_integer-cpu-arguments-constant"],
                            cost_map["divide_integer-cpu-arguments-model-arguments-intercept"],
                            cost_map["divide_integer-cpu-arguments-model-arguments-slope"],
                        )
                    }
                    PlutusVersion::V3 => {
                        TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                            cost_map["divide_integer-cpu-arguments-constant"],
                            cost_map["divide_integer-cpu-arguments-minimum"],
                            cost_map["divide_integer-cpu-arguments-c00"],
                            cost_map["divide_integer-cpu-arguments-c01"],
                            cost_map["divide_integer-cpu-arguments-c02"],
                            cost_map["divide_integer-cpu-arguments-c10"],
                            cost_map["divide_integer-cpu-arguments-c11"],
                            cost_map["divide_integer-cpu-arguments-c20"],
                        )
                    }
                },
            ),
            encode_utf8: OneArgumentCosting::new(
                OneArgumentCosting::linear_cost(
                    cost_map["encode_utf8-mem-arguments-intercept"],
                    cost_map["encode_utf8-mem-arguments-slope"],
                ),
                OneArgumentCosting::linear_cost(
                    cost_map["encode_utf8-cpu-arguments-intercept"],
                    cost_map["encode_utf8-cpu-arguments-slope"],
                ),
            ),
            equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["equals_byte_string-mem-arguments"]),
                TwoArgumentsCosting::linear_on_diagonal(
                    cost_map["equals_byte_string-cpu-arguments-constant"],
                    cost_map["equals_byte_string-cpu-arguments-intercept"],
                    cost_map["equals_byte_string-cpu-arguments-slope"],
                ),
            ),
            equals_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["equals_data-mem-arguments"]),
                TwoArgumentsCosting::min_size(
                    cost_map["equals_data-cpu-arguments-intercept"],
                    cost_map["equals_data-cpu-arguments-slope"],
                ),
            ),
            equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["equals_integer-mem-arguments"]),
                TwoArgumentsCosting::min_size(
                    cost_map["equals_integer-cpu-arguments-intercept"],
                    cost_map["equals_integer-cpu-arguments-slope"],
                ),
            ),
            equals_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["equals_string-mem-arguments"]),
                TwoArgumentsCosting::linear_on_diagonal(
                    cost_map["equals_string-cpu-arguments-constant"],
                    cost_map["equals_string-cpu-arguments-intercept"],
                    cost_map["equals_string-cpu-arguments-slope"],
                ),
            ),
            fst_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["fst_pair-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["fst_pair-cpu-arguments"]),
            ),
            head_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["head_list-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["head_list-cpu-arguments"]),
            ),
            i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["i_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["i_data-cpu-arguments"]),
            ),
            if_then_else: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(cost_map["if_then_else-mem-arguments"]),
                ThreeArgumentsCosting::constant_cost(cost_map["if_then_else-cpu-arguments"]),
            ),
            index_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["index_byte_string-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["index_byte_string-cpu-arguments"]),
            ),
            length_of_byte_string: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["length_of_byte_string-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["length_of_byte_string-cpu-arguments"]),
            ),
            less_than_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["less_than_byte_string-mem-arguments"]),
                TwoArgumentsCosting::min_size(
                    cost_map["less_than_byte_string-cpu-arguments-intercept"],
                    cost_map["less_than_byte_string-cpu-arguments-slope"],
                ),
            ),
            less_than_equals_byte_string: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(
                    cost_map["less_than_equals_byte_string-mem-arguments"],
                ),
                TwoArgumentsCosting::min_size(
                    cost_map["less_than_equals_byte_string-cpu-arguments-intercept"],
                    cost_map["less_than_equals_byte_string-cpu-arguments-slope"],
                ),
            ),
            less_than_equals_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(
                    cost_map["less_than_equals_integer-mem-arguments"],
                ),
                TwoArgumentsCosting::min_size(
                    cost_map["less_than_equals_integer-cpu-arguments-intercept"],
                    cost_map["less_than_equals_integer-cpu-arguments-slope"],
                ),
            ),
            less_than_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["less_than_integer-mem-arguments"]),
                TwoArgumentsCosting::min_size(
                    cost_map["less_than_integer-cpu-arguments-intercept"],
                    cost_map["less_than_integer-cpu-arguments-slope"],
                ),
            ),
            list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["list_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["list_data-cpu-arguments"]),
            ),
            map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["map_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["map_data-cpu-arguments"]),
            ),
            mk_cons: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["mk_cons-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["mk_cons-cpu-arguments"]),
            ),
            mk_nil_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["mk_nil_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["mk_nil_data-cpu-arguments"]),
            ),
            mk_nil_pair_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["mk_nil_pair_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["mk_nil_pair_data-cpu-arguments"]),
            ),
            mk_pair_data: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["mk_pair_data-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["mk_pair_data-cpu-arguments"]),
            ),
            mod_integer: TwoArgumentsCosting::new(
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::subtracted_sizes(
                        cost_map["mod_integer-mem-arguments-intercept"],
                        cost_map["mod_integer-mem-arguments-slope"],
                        cost_map["mod_integer-mem-arguments-minimum"],
                    ),
                    PlutusVersion::V3 => TwoArgumentsCosting::linear_in_y(
                        cost_map["mod_integer-mem-arguments-intercept"],
                        cost_map["mod_integer-mem-arguments-slope"],
                    ),
                },
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => {
                        TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(
                            cost_map["mod_integer-cpu-arguments-constant"],
                            cost_map["mod_integer-cpu-arguments-model-arguments-intercept"],
                            cost_map["mod_integer-cpu-arguments-model-arguments-slope"],
                        )
                    }
                    PlutusVersion::V3 => {
                        TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                            cost_map["mod_integer-cpu-arguments-constant"],
                            cost_map["mod_integer-cpu-arguments-minimum"],
                            cost_map["mod_integer-cpu-arguments-c00"],
                            cost_map["mod_integer-cpu-arguments-c01"],
                            cost_map["mod_integer-cpu-arguments-c02"],
                            cost_map["mod_integer-cpu-arguments-c10"],
                            cost_map["mod_integer-cpu-arguments-c11"],
                            cost_map["mod_integer-cpu-arguments-c20"],
                        )
                    }
                },
            ),
            multiply_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::added_sizes(
                    cost_map["multiply_integer-mem-arguments-intercept"],
                    cost_map["multiply_integer-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::multiplied_sizes(
                    cost_map["multiply_integer-cpu-arguments-intercept"],
                    cost_map["multiply_integer-cpu-arguments-slope"],
                ),
            ),
            null_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["null_list-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["null_list-cpu-arguments"]),
            ),
            quotient_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::subtracted_sizes(
                    cost_map["quotient_integer-mem-arguments-intercept"],
                    cost_map["quotient_integer-mem-arguments-slope"],
                    cost_map["quotient_integer-mem-arguments-minimum"],
                ),
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => {
                        TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(
                            cost_map["quotient_integer-cpu-arguments-constant"],
                            cost_map["quotient_integer-cpu-arguments-model-arguments-intercept"],
                            cost_map["quotient_integer-cpu-arguments-model-arguments-slope"],
                        )
                    }
                    PlutusVersion::V3 => {
                        TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                            cost_map["quotient_integer-cpu-arguments-constant"],
                            cost_map["quotient_integer-cpu-arguments-minimum"],
                            cost_map["quotient_integer-cpu-arguments-c00"],
                            cost_map["quotient_integer-cpu-arguments-c01"],
                            cost_map["quotient_integer-cpu-arguments-c02"],
                            cost_map["quotient_integer-cpu-arguments-c10"],
                            cost_map["quotient_integer-cpu-arguments-c11"],
                            cost_map["quotient_integer-cpu-arguments-c20"],
                        )
                    }
                },
            ),
            remainder_integer: TwoArgumentsCosting::new(
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::subtracted_sizes(
                        cost_map["remainder_integer-mem-arguments-intercept"],
                        cost_map["remainder_integer-mem-arguments-slope"],
                        cost_map["remainder_integer-mem-arguments-minimum"],
                    ),
                    PlutusVersion::V3 => TwoArgumentsCosting::linear_in_y(
                        cost_map["remainder_integer-mem-arguments-intercept"],
                        cost_map["remainder_integer-mem-arguments-slope"],
                    ),
                },
                match version {
                    PlutusVersion::V1 | PlutusVersion::V2 => {
                        TwoArgumentsCosting::const_above_diagonal_into_multiplied_sizes(
                            cost_map["remainder_integer-cpu-arguments-constant"],
                            cost_map["remainder_integer-cpu-arguments-model-arguments-intercept"],
                            cost_map["remainder_integer-cpu-arguments-model-arguments-slope"],
                        )
                    }
                    PlutusVersion::V3 => {
                        TwoArgumentsCosting::const_above_diagonal_into_quadratic_x_and_y(
                            cost_map["remainder_integer-cpu-arguments-constant"],
                            cost_map["remainder_integer-cpu-arguments-minimum"],
                            cost_map["remainder_integer-cpu-arguments-c00"],
                            cost_map["remainder_integer-cpu-arguments-c01"],
                            cost_map["remainder_integer-cpu-arguments-c02"],
                            cost_map["remainder_integer-cpu-arguments-c10"],
                            cost_map["remainder_integer-cpu-arguments-c11"],
                            cost_map["remainder_integer-cpu-arguments-c20"],
                        )
                    }
                },
            ),
            serialise_data: match version {
                PlutusVersion::V1 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V2 | PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::linear_cost(
                        cost_map["serialise_data-mem-arguments-intercept"],
                        cost_map["serialise_data-mem-arguments-slope"],
                    ),
                    OneArgumentCosting::linear_cost(
                        cost_map["serialise_data-cpu-arguments-intercept"],
                        cost_map["serialise_data-cpu-arguments-slope"],
                    ),
                ),
            },
            sha2_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["sha2_256-mem-arguments"]),
                OneArgumentCosting::linear_cost(
                    cost_map["sha2_256-cpu-arguments-intercept"],
                    cost_map["sha2_256-cpu-arguments-slope"],
                ),
            ),
            sha3_256: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["sha3_256-mem-arguments"]),
                OneArgumentCosting::linear_cost(
                    cost_map["sha3_256-cpu-arguments-intercept"],
                    cost_map["sha3_256-cpu-arguments-slope"],
                ),
            ),
            slice_byte_string: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::linear_in_z(
                    cost_map["slice_byte_string-mem-arguments-intercept"],
                    cost_map["slice_byte_string-mem-arguments-slope"],
                ),
                ThreeArgumentsCosting::linear_in_z(
                    cost_map["slice_byte_string-cpu-arguments-intercept"],
                    cost_map["slice_byte_string-cpu-arguments-slope"],
                ),
            ),
            snd_pair: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["snd_pair-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["snd_pair-cpu-arguments"]),
            ),
            subtract_integer: TwoArgumentsCosting::new(
                TwoArgumentsCosting::max_size(
                    cost_map["subtract_integer-mem-arguments-intercept"],
                    cost_map["subtract_integer-mem-arguments-slope"],
                ),
                TwoArgumentsCosting::max_size(
                    cost_map["subtract_integer-cpu-arguments-intercept"],
                    cost_map["subtract_integer-cpu-arguments-slope"],
                ),
            ),
            tail_list: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["tail_list-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["tail_list-cpu-arguments"]),
            ),
            trace: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(cost_map["trace-mem-arguments"]),
                TwoArgumentsCosting::constant_cost(cost_map["trace-cpu-arguments"]),
            ),
            un_b_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["un_b_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["un_b_data-cpu-arguments"]),
            ),
            un_constr_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["un_constr_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["un_constr_data-cpu-arguments"]),
            ),
            un_i_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["un_i_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["un_i_data-cpu-arguments"]),
            ),
            un_list_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["un_list_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["un_list_data-cpu-arguments"]),
            ),
            un_map_data: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(cost_map["un_map_data-mem-arguments"]),
                OneArgumentCosting::constant_cost(cost_map["un_map_data-cpu-arguments"]),
            ),
            verify_ecdsa_secp256k1_signature: match version {
                PlutusVersion::V1 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V2 | PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(
                        cost_map["verify_ecdsa_secp256k1_signature-mem-arguments"],
                    ),
                    ThreeArgumentsCosting::constant_cost(
                        cost_map["verify_ecdsa_secp256k1_signature-cpu-arguments"],
                    ),
                ),
            },
            verify_ed25519_signature: ThreeArgumentsCosting::new(
                ThreeArgumentsCosting::constant_cost(
                    cost_map["verify_ed25519_signature-mem-arguments"],
                ),
                ThreeArgumentsCosting::linear_in_y(
                    cost_map["verify_ed25519_signature-cpu-arguments-intercept"],
                    cost_map["verify_ed25519_signature-cpu-arguments-slope"],
                ),
            ),
            verify_schnorr_secp256k1_signature: match version {
                PlutusVersion::V1 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V2 | PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(
                        cost_map["verify_schnorr_secp256k1_signature-mem-arguments"],
                    ),
                    ThreeArgumentsCosting::linear_in_y(
                        cost_map["verify_schnorr_secp256k1_signature-cpu-arguments-intercept"],
                        cost_map["verify_schnorr_secp256k1_signature-cpu-arguments-slope"],
                    ),
                ),
            },
            bls12_381_g1_add: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(cost_map["bls12_381_G1_add-mem-arguments"]),
                    TwoArgumentsCosting::constant_cost(cost_map["bls12_381_G1_add-cpu-arguments"]),
                ),
            },
            bls12_381_g1_compress: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G1_compress-mem-arguments"],
                    ),
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G1_compress-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_g1_equal: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G1_equal-mem-arguments"],
                    ),
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G1_equal-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_g1_hash_to_group: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G1_hashToGroup-mem-arguments"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["bls12_381_G1_hashToGroup-cpu-arguments-intercept"],
                        cost_map["bls12_381_G1_hashToGroup-cpu-arguments-slope"],
                    ),
                ),
            },
            bls12_381_g1_neg: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["bls12_381_G1_neg-mem-arguments"]),
                    OneArgumentCosting::constant_cost(cost_map["bls12_381_G1_neg-cpu-arguments"]),
                ),
            },
            bls12_381_g1_scalar_mul: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G1_scalarMul-mem-arguments"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["bls12_381_G1_scalarMul-cpu-arguments-intercept"],
                        cost_map["bls12_381_G1_scalarMul-cpu-arguments-slope"],
                    ),
                ),
            },
            bls12_381_g1_uncompress: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G1_uncompress-mem-arguments"],
                    ),
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G1_uncompress-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_g2_add: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(cost_map["bls12_381_G2_add-mem-arguments"]),
                    TwoArgumentsCosting::constant_cost(cost_map["bls12_381_G2_add-cpu-arguments"]),
                ),
            },
            bls12_381_g2_compress: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G2_compress-mem-arguments"],
                    ),
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G2_compress-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_g2_equal: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G2_equal-mem-arguments"],
                    ),
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G2_equal-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_g2_hash_to_group: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G2_hashToGroup-mem-arguments"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["bls12_381_G2_hashToGroup-cpu-arguments-intercept"],
                        cost_map["bls12_381_G2_hashToGroup-cpu-arguments-slope"],
                    ),
                ),
            },
            bls12_381_g2_neg: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["bls12_381_G2_neg-mem-arguments"]),
                    OneArgumentCosting::constant_cost(cost_map["bls12_381_G2_neg-cpu-arguments"]),
                ),
            },
            bls12_381_g2_scalar_mul: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_G2_scalarMul-mem-arguments"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["bls12_381_G2_scalarMul-cpu-arguments-intercept"],
                        cost_map["bls12_381_G2_scalarMul-cpu-arguments-slope"],
                    ),
                ),
            },
            bls12_381_g2_uncompress: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G2_uncompress-mem-arguments"],
                    ),
                    OneArgumentCosting::constant_cost(
                        cost_map["bls12_381_G2_uncompress-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_final_verify: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_finalVerify-mem-arguments"],
                    ),
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_finalVerify-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_miller_loop: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_millerLoop-mem-arguments"],
                    ),
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_millerLoop-cpu-arguments"],
                    ),
                ),
            },
            bls12_381_mul_ml_result: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_mulMlResult-mem-arguments"],
                    ),
                    TwoArgumentsCosting::constant_cost(
                        cost_map["bls12_381_mulMlResult-cpu-arguments"],
                    ),
                ),
            },
            keccak_256: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["keccak_256-mem-arguments"]),
                    OneArgumentCosting::linear_cost(
                        cost_map["keccak_256-cpu-arguments-intercept"],
                        cost_map["keccak_256-cpu-arguments-slope"],
                    ),
                ),
            },
            blake2b_224: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["blake2b_224-mem-arguments-slope"]),
                    OneArgumentCosting::linear_cost(
                        cost_map["blake2b_224-cpu-arguments-intercept"],
                        cost_map["blake2b_224-cpu-arguments-slope"],
                    ),
                ),
            },
            integer_to_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::literal_in_y_or_linear_in_z(
                        cost_map["integerToByteString-mem-arguments-intercept"],
                        cost_map["integerToByteString-mem-arguments-slope"],
                    ),
                    ThreeArgumentsCosting::quadratic_in_z(
                        cost_map["integerToByteString-cpu-arguments-c0"],
                        cost_map["integerToByteString-cpu-arguments-c1"],
                        cost_map["integerToByteString-cpu-arguments-c2"],
                    ),
                ),
            },
            byte_string_to_integer: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::linear_in_y(
                        cost_map["byteStringToInteger-mem-arguments-intercept"],
                        cost_map["byteStringToInteger-mem-arguments-slope"],
                    ),
                    TwoArgumentsCosting::quadratic_in_y(
                        cost_map["byteStringToInteger-cpu-arguments-c0"],
                        cost_map["byteStringToInteger-cpu-arguments-c1"],
                        cost_map["byteStringToInteger-cpu-arguments-c2"],
                    ),
                ),
            },
            and_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::linear_in_max_y_z(
                        cost_map["andByteString-memory-arguments-intercept"],
                        cost_map["andByteString-memory-arguments-slope"],
                    ),
                    ThreeArgumentsCosting::linear_in_y_and_z(
                        cost_map["andByteString-cpu-arguments-intercept"],
                        cost_map["andByteString-cpu-arguments-slope1"],
                        cost_map["andByteString-cpu-arguments-slope2"],
                    ),
                ),
            },
            or_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::linear_in_max_y_z(
                        cost_map["orByteString-memory-arguments-intercept"],
                        cost_map["orByteString-memory-arguments-slope"],
                    ),
                    ThreeArgumentsCosting::linear_in_y_and_z(
                        cost_map["orByteString-cpu-arguments-intercept"],
                        cost_map["orByteString-cpu-arguments-slope1"],
                        cost_map["orByteString-cpu-arguments-slope2"],
                    ),
                ),
            },
            xor_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::linear_in_max_y_z(
                        cost_map["xorByteString-memory-arguments-intercept"],
                        cost_map["xorByteString-memory-arguments-slope"],
                    ),
                    ThreeArgumentsCosting::linear_in_y_and_z(
                        cost_map["xorByteString-cpu-arguments-intercept"],
                        cost_map["xorByteString-cpu-arguments-slope1"],
                        cost_map["xorByteString-cpu-arguments-slope2"],
                    ),
                ),
            },
            complement_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::linear_cost(
                        cost_map["complementByteString-memory-arguments-intercept"],
                        cost_map["complementByteString-memory-arguments-slope"],
                    ),
                    OneArgumentCosting::linear_cost(
                        cost_map["complementByteString-cpu-arguments-intercept"],
                        cost_map["complementByteString-cpu-arguments-slope"],
                    ),
                ),
            },
            read_bit: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(cost_map["readBit-memory-arguments"]),
                    TwoArgumentsCosting::constant_cost(cost_map["readBit-cpu-arguments"]),
                ),
            },
            write_bits: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::linear_in_x(
                        cost_map["writeBits-memory-arguments-intercept"],
                        cost_map["writeBits-memory-arguments-slope"],
                    ),
                    ThreeArgumentsCosting::linear_in_y(
                        cost_map["writeBits-cpu-arguments-intercept"],
                        cost_map["writeBits-cpu-arguments-slope"],
                    ),
                ),
            },
            replicate_byte: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["replicateByte-memory-arguments-intercept"],
                        cost_map["replicateByte-memory-arguments-slope"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["replicateByte-cpu-arguments-intercept"],
                        cost_map["replicateByte-cpu-arguments-slope"],
                    ),
                ),
            },
            shift_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["shiftByteString-memory-arguments-intercept"],
                        cost_map["shiftByteString-memory-arguments-slope"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["shiftByteString-cpu-arguments-intercept"],
                        cost_map["shiftByteString-cpu-arguments-slope"],
                    ),
                ),
            },
            rotate_byte_string: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::constant_cost(30000000000),
                    TwoArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => TwoArgumentsCosting::new(
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["rotateByteString-memory-arguments-intercept"],
                        cost_map["rotateByteString-memory-arguments-slope"],
                    ),
                    TwoArgumentsCosting::linear_in_x(
                        cost_map["rotateByteString-cpu-arguments-intercept"],
                        cost_map["rotateByteString-cpu-arguments-slope"],
                    ),
                ),
            },
            count_set_bits: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["countSetBits-memory-arguments"]),
                    OneArgumentCosting::linear_cost(
                        cost_map["countSetBits-cpu-arguments-intercept"],
                        cost_map["countSetBits-cpu-arguments-slope"],
                    ),
                ),
            },
            find_first_set_bit: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["findFirstSetBit-memory-arguments"]),
                    OneArgumentCosting::linear_cost(
                        cost_map["findFirstSetBit-cpu-arguments-intercept"],
                        cost_map["findFirstSetBit-cpu-arguments-slope"],
                    ),
                ),
            },
            ripemd_160: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(30000000000),
                    OneArgumentCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => OneArgumentCosting::new(
                    OneArgumentCosting::constant_cost(cost_map["ripemd_160-memory-arguments"]),
                    OneArgumentCosting::linear_cost(
                        cost_map["ripemd_160-cpu-arguments-intercept"],
                        cost_map["ripemd_160-cpu-arguments-slope"],
                    ),
                ),
            },
            exp_mod_integer: match version {
                PlutusVersion::V1 | PlutusVersion::V2 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::constant_cost(30000000000),
                    ThreeArgumentsCosting::constant_cost(30000000000),
                ),
                PlutusVersion::V3 => ThreeArgumentsCosting::new(
                    ThreeArgumentsCosting::linear_in_z(0, 1),
                    ThreeArgumentsCosting::exp_mod_cost(607153, 231697, 53144),
                ),
            },
            drop_list: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(4),
                TwoArgumentsCosting::linear_in_x(116711, 1957),
            ),
            length_of_array: OneArgumentCosting::new(
                OneArgumentCosting::constant_cost(10),
                OneArgumentCosting::constant_cost(198994),
            ),
            list_to_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::linear_in_x(7, 1),
                TwoArgumentsCosting::linear_in_x(307802, 8496),
            ),
            index_array: TwoArgumentsCosting::new(
                TwoArgumentsCosting::constant_cost(32),
                TwoArgumentsCosting::constant_cost(194922),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn assert_default_cost_model_v1() {
        let costs = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 228465, 122, 0, 1, 1, 1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148,
            27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32,
            76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541,
            1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 228465, 122,
            0, 1, 1, 90434, 519, 0, 1, 74433, 32, 85848, 228465, 122, 0, 1, 1, 85848, 228465, 122,
            0, 1, 1, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788, 420,
            1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32, 25933, 32, 24623, 32,
            53384111, 14333, 10,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V1, &costs);

        assert_eq!(
            BuiltinCosts::v1(),
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V1, &cost_model)
        );
    }

    #[test]
    fn assert_default_cost_model_v2() {
        let costs = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 228465, 122, 0, 1, 1, 1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148,
            27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32,
            76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541,
            1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 228465, 122,
            0, 1, 1, 90434, 519, 0, 1, 74433, 32, 85848, 228465, 122, 0, 1, 1, 85848, 228465, 122,
            0, 1, 1, 955506, 213312, 0, 2, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0,
            141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32,
            25933, 32, 24623, 32, 43053543, 10, 53384111, 14333, 10, 43574283, 26308, 10,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V2, &costs);

        assert_eq!(
            BuiltinCosts::v2(),
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V2, &cost_model)
        );
    }

    #[test]
    fn assert_default_cost_model_v3() {
        let costs: Vec<i64> = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1, 1000, 42921, 4, 2,
            24548, 29498, 38, 1, 898148, 27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895,
            32, 83150, 32, 15299, 32, 76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1,
            43285, 552, 1, 44749, 541, 1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32,
            11546, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 90434, 519, 0, 1,
            74433, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1, 85848, 123203,
            7305, -900, 1716, 549, 57, 85848, 0, 1, 955506, 213312, 0, 2, 270652, 22588, 4,
            1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32,
            20142, 32, 24588, 32, 20744, 32, 25933, 32, 24623, 32, 43053543, 10, 53384111, 14333,
            10, 43574283, 26308, 10, 16000, 100, 16000, 100, 962335, 18, 2780678, 6, 442008, 1,
            52538055, 3756, 18, 267929, 18, 76433006, 8868, 18, 52948122, 18, 1995836, 36, 3227919,
            12, 901022, 1, 166917843, 4307, 36, 284546, 36, 158221314, 26549, 36, 74698472, 36,
            333849714, 1, 254006273, 72, 2174038, 72, 2261318, 64571, 4, 207616, 8310, 4, 1293828,
            28716, 63, 0, 1, 1006041, 43623, 251, 0, 1, 100181, 726, 719, 0, 1, 100181, 726, 719,
            0, 1, 100181, 726, 719, 0, 1, 107878, 680, 0, 1, 95336, 1, 281145, 18848, 0, 1, 180194,
            159, 1, 1, 158519, 8942, 0, 1, 159378, 8813, 0, 1, 107490, 3298, 1, 106057, 655, 1,
            1964219, 24520, 3,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V3, &costs);

        assert_eq!(
            BuiltinCosts::v3(),
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V3, &cost_model)
        );
    }
}
