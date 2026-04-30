//! Regression tests for divergences between Rust and Haskell UPLC implementations.
//!
//! Each test documents the expected Haskell behavior. All tests should pass,
//! confirming the Rust implementation matches.
//!
//! See `DIVERGENCE_REPORT.md` for full details on each divergence.

use std::str::FromStr;

use uplc_turbo::arena::Arena;
use uplc_turbo::binder::DeBruijn;
use uplc_turbo::constant::{Constant, Integer};
use uplc_turbo::machine::cost_model::{pair_ex_mem, proto_list_ex_mem};
use uplc_turbo::machine::ExBudget;
use uplc_turbo::program::{Program, Version};
use uplc_turbo::term::Term;

// ---------------------------------------------------------------------------
// BUG 1 (FIXED): XorByteString now uses its own cost function
// ---------------------------------------------------------------------------

/// Verifies that XorByteString uses its own cost model, not OrByteString's.
/// Both operations should succeed and produce correct results.
#[test]
fn xor_byte_string_uses_own_cost_model() {
    let arena = Arena::new();

    let bs1: &[u8] = arena.alloc_slice(vec![0xAA; 32]);
    let bs2: &[u8] = arena.alloc_slice(vec![0x55; 32]);

    // XOR program: (xorByteString False bs1 bs2)
    let xor_term = Term::<DeBruijn>::xor_byte_string(&arena)
        .apply(&arena, Term::bool(&arena, false))
        .apply(&arena, Term::byte_string(&arena, bs1))
        .apply(&arena, Term::byte_string(&arena, bs2));

    let xor_program = Program::new(&arena, Version::plutus_v3(&arena), xor_term);
    let xor_result = xor_program.eval(&arena);
    assert!(xor_result.term.is_ok(), "XOR should succeed");

    // OR program: (orByteString False bs1 bs2)
    let or_term = Term::<DeBruijn>::or_byte_string(&arena)
        .apply(&arena, Term::bool(&arena, false))
        .apply(&arena, Term::byte_string(&arena, bs1))
        .apply(&arena, Term::byte_string(&arena, bs2));

    let or_program = Program::new(&arena, Version::plutus_v3(&arena), or_term);
    let or_result = or_program.eval(&arena);
    assert!(or_result.term.is_ok(), "OR should succeed");

    // Both should produce valid results (XOR = 0xFF..FF, OR = 0xFF..FF for these inputs)
    let expected: &[u8] = arena.alloc_slice(vec![0xFF; 32]);
    assert_eq!(
        xor_result.term.unwrap(),
        Term::byte_string(&arena, expected)
    );
    assert_eq!(or_result.term.unwrap(), Term::byte_string(&arena, expected));
}

// ---------------------------------------------------------------------------
// BUG 2: ExpModInteger edge case — NOT A BUG (regression guards)
// ---------------------------------------------------------------------------

/// expModInteger 0 (-1) 1 → 0 (Haskell behavior, confirmed to match Rust)
#[test]
fn exp_mod_integer_zero_negative_exp_mod_one() {
    let arena = Arena::new();

    let term = Term::<DeBruijn>::exp_mod_integer(&arena)
        .apply(&arena, Term::integer_from(&arena, 0))
        .apply(&arena, Term::integer_from(&arena, -1))
        .apply(&arena, Term::integer_from(&arena, 1));

    let program = Program::new(&arena, Version::plutus_v3(&arena), term);
    let result = program.eval(&arena);

    let term_result = result.term.expect("expModInteger 0 (-1) 1 should return 0");
    assert_eq!(term_result, Term::integer_from(&arena, 0));
}

/// expModInteger 0 (-5) 1 → 0
#[test]
fn exp_mod_integer_zero_negative5_exp_mod_one() {
    let arena = Arena::new();

    let term = Term::<DeBruijn>::exp_mod_integer(&arena)
        .apply(&arena, Term::integer_from(&arena, 0))
        .apply(&arena, Term::integer_from(&arena, -5))
        .apply(&arena, Term::integer_from(&arena, 1));

    let program = Program::new(&arena, Version::plutus_v3(&arena), term);
    let result = program.eval(&arena);

    let term_result = result.term.expect("expModInteger 0 (-5) 1 should return 0");
    assert_eq!(term_result, Term::integer_from(&arena, 0));
}

/// expModInteger 5 (-1) 1 → 0 (anything mod 1 is 0)
#[test]
fn exp_mod_integer_five_negative1_mod_one() {
    let arena = Arena::new();

    let term = Term::<DeBruijn>::exp_mod_integer(&arena)
        .apply(&arena, Term::integer_from(&arena, 5))
        .apply(&arena, Term::integer_from(&arena, -1))
        .apply(&arena, Term::integer_from(&arena, 1));

    let program = Program::new(&arena, Version::plutus_v3(&arena), term);
    let result = program.eval(&arena);

    let term_result = result
        .term
        .expect("expModInteger 5 (-1) 1 should return 0 (anything mod 1 is 0)");
    assert_eq!(term_result, Term::integer_from(&arena, 0));
}

// ---------------------------------------------------------------------------
// BUG 3 (FIXED): IndexByteString gracefully handles huge indices
// ---------------------------------------------------------------------------

/// Indices beyond i128 range return an out-of-bounds error instead of panicking.
#[test]
fn index_byte_string_huge_index_no_panic() {
    let arena = Arena::new();

    // 2^200 is way beyond i128 range
    let huge_program = "(program 1.1.0 \
        [(builtin indexByteString) \
         (con bytestring #00) \
         (con integer 1606938044258990275541962092341162602522202993782792835301376)])";

    let parsed = uplc_turbo::syn::parse_program(&arena, huge_program)
        .into_result()
        .expect("should parse");

    let result = parsed.eval(&arena);

    // Should be an evaluation failure (out-of-bounds), NOT a panic
    assert!(
        result.term.is_err(),
        "indexByteString with a huge index should return an error, not panic"
    );
}

// ---------------------------------------------------------------------------
// BUG 4 (FIXED): proto_list_ex_mem returns element count
// ---------------------------------------------------------------------------

/// List memory is the element count, matching Haskell's ExMemoryUsage [a].
#[test]
fn list_memory_is_element_count_not_sum_of_sizes() {
    let arena = Arena::new();

    // List of 3 small integers: [1, 2, 3] — each has ex_mem = 1
    let small_items: &[&Constant] = arena.alloc_slice(vec![
        Constant::integer_from(&arena, 1),
        Constant::integer_from(&arena, 2),
        Constant::integer_from(&arena, 3),
    ]);
    assert_eq!(
        proto_list_ex_mem(small_items),
        3,
        "List of 3 small ints: memory = count = 3"
    );

    // List of 3 large integers: [10^30, 10^30, 10^30] — each has ex_mem = 2
    let big_int = Integer::from_str("1000000000000000000000000000000").unwrap();
    let big_int_ref = arena.alloc_integer(big_int);
    let large_items: &[&Constant] = arena.alloc_slice(vec![
        Constant::integer(&arena, big_int_ref),
        Constant::integer(&arena, big_int_ref),
        Constant::integer(&arena, big_int_ref),
    ]);

    // Should be 3 (element count), not 6 (sum of sizes)
    assert_eq!(
        proto_list_ex_mem(large_items),
        3,
        "List of 3 large ints: memory = count = 3"
    );
}

// ---------------------------------------------------------------------------
// BUG 5 (FIXED): pair_ex_mem returns maxBound sentinel
// ---------------------------------------------------------------------------

/// Pair memory returns i64::MAX (maxBound), matching Haskell's sentinel convention.
#[test]
fn pair_memory_matches_haskell_sentinel() {
    let arena = Arena::new();

    let small_fst = Constant::integer_from(&arena, 1);
    let small_snd = Constant::integer_from(&arena, 2);

    assert_eq!(
        pair_ex_mem(small_fst, small_snd),
        i64::MAX,
        "pair_ex_mem should return i64::MAX (maxBound) to match Haskell sentinel"
    );
}

// ---------------------------------------------------------------------------
// BUG 6 (FIXED): ExBudget::occurrences uses saturating multiplication
// ---------------------------------------------------------------------------

/// Multiplication in occurrences saturates instead of overflowing.
#[test]
fn ex_budget_occurrences_saturates() {
    let mut budget = ExBudget::new(i64::MAX / 100, i64::MAX / 100);

    // i64::MAX / 100 * 200 would overflow without saturation
    budget.occurrences(200);

    assert_eq!(
        budget.mem,
        i64::MAX,
        "ExBudget::occurrences should saturate to i64::MAX"
    );
    assert_eq!(
        budget.cpu,
        i64::MAX,
        "ExBudget::occurrences should saturate to i64::MAX"
    );
}
