//! Tests for case expressions on boolean values.
//!
//! In Plutus V3, booleans from builtins (equalsInteger, lessThanInteger, etc.)
//! are returned as Con(Boolean(true/false)). However, case expressions expect
//! Constr values. The Haskell reference implementation handles this — uplc-turbo
//! currently fails with NonConstrScrutinized.
//!
//! These tests document the expected behavior. They currently FAIL and should
//! start passing once the fix is implemented.

use uplc_turbo::{arena::Arena, machine::PlutusVersion, syn};

fn eval_v3(source: &str) -> Result<String, String> {
    let arena = Arena::new();
    let program = syn::parse_program(&arena, source)
        .into_result()
        .map_err(|_| "parse error".to_string())?;

    let result = program.eval_version(&arena, PlutusVersion::V3);
    match result.term {
        Ok(term) => Ok(format!("{:?}", term)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// === case on boolean from builtin ===

#[test]
fn case_on_equals_integer_true() {
    // equalsInteger 1 1 = True → case selects branch at index 1
    // Bool in Plutus: False = constr 0, True = constr 1
    let result = eval_v3(
        "(program 1.1.0 (case [(builtin equalsInteger) (con integer 1) (con integer 1)] (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(1)"), "expected 1 (true branch), got: {result}");
}

#[test]
fn case_on_equals_integer_false() {
    let result = eval_v3(
        "(program 1.1.0 (case [(builtin equalsInteger) (con integer 1) (con integer 2)] (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(0)"), "expected 0 (false branch), got: {result}");
}

#[test]
fn case_on_less_than_integer() {
    let result = eval_v3(
        "(program 1.1.0 (case [(builtin lessThanInteger) (con integer 1) (con integer 2)] (con integer 10) (con integer 20)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(20)"), "expected 20 (true branch), got: {result}");
}

#[test]
fn case_on_null_list() {
    let result = eval_v3(
        "(program 1.1.0 (case [(force (builtin nullList)) (con (list integer) [])] (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(1)"), "expected 1 (true = empty list), got: {result}");
}

#[test]
fn case_on_equals_bytestring() {
    let result = eval_v3(
        "(program 1.1.0 (case [(builtin equalsByteString) (con bytestring #deadbeef) (con bytestring #deadbeef)] (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(1)"), "expected 1 (true), got: {result}");
}

// === case on boolean constant ===

#[test]
fn case_on_bool_constant_true() {
    let result = eval_v3(
        "(program 1.1.0 (case (con bool True) (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(1)"), "expected 1 (true branch), got: {result}");
}

#[test]
fn case_on_bool_constant_false() {
    let result = eval_v3(
        "(program 1.1.0 (case (con bool False) (con integer 0) (con integer 1)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(0)"), "expected 0 (false branch), got: {result}");
}

// === case on constr (sanity checks — these should already work) ===

#[test]
fn case_on_constr_tag0() {
    let result = eval_v3(
        "(program 1.1.0 (case (constr 0) (con integer 10) (con integer 20)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(10)"), "expected 10 (tag 0), got: {result}");
}

#[test]
fn case_on_constr_tag1() {
    let result = eval_v3(
        "(program 1.1.0 (case (constr 1) (con integer 10) (con integer 20)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(20)"), "expected 20 (tag 1), got: {result}");
}

#[test]
fn case_on_constr_with_fields() {
    let result = eval_v3(
        "(program 1.1.0 (case (constr 1 (con integer 42)) (lam x (con integer 0)) (lam x x)))"
    ).expect("should evaluate successfully");
    assert!(result.contains("Integer(42)"), "expected 42, got: {result}");
}
