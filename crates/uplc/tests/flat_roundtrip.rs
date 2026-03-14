use uplc_turbo::{arena::Arena, binder::DeBruijn, flat, syn};

/// Parse a UPLC program from text, encode to FLAT, decode back, and verify
/// the result matches.
fn roundtrip(source: &str) {
    let arena = Arena::new();
    let program = syn::parse_program(&arena, source)
        .into_result()
        .unwrap_or_else(|_| panic!("Failed to parse: {source}"));

    let encoded = flat::encode(program).unwrap_or_else(|e| {
        panic!("Failed to encode: {e:?}\nSource: {source}")
    });

    let arena2 = Arena::new();
    let decoded = flat::decode::<DeBruijn>(&arena2, &encoded).unwrap_or_else(|e| {
        panic!("Failed to decode: {e:?}\nSource: {source}")
    });

    assert_eq!(
        program.term, decoded.term,
        "Round-trip mismatch for: {source}"
    );
}

/// Parse from text, encode to FLAT, decode, evaluate, and check it succeeds.
fn roundtrip_eval(source: &str) {
    let arena = Arena::new();
    let program = syn::parse_program(&arena, source)
        .into_result()
        .unwrap_or_else(|_| panic!("Failed to parse: {source}"));

    let encoded = flat::encode(program).unwrap_or_else(|e| {
        panic!("Failed to encode: {e:?}\nSource: {source}")
    });

    let arena2 = Arena::new();
    let decoded = flat::decode::<DeBruijn>(&arena2, &encoded).unwrap_or_else(|e| {
        panic!("Failed to decode: {e:?}\nSource: {source}")
    });

    let result = decoded.eval(&arena2);
    result
        .term
        .unwrap_or_else(|e| panic!("Failed to evaluate: {e:?}\nSource: {source}"));
}

// --- Basic term round-trips ---

#[test]
fn flat_roundtrip_integer() {
    roundtrip("(program 1.0.0 (con integer 42))");
}

#[test]
fn flat_roundtrip_negative_integer() {
    roundtrip("(program 1.0.0 (con integer -123456789))");
}

#[test]
fn flat_roundtrip_big_integer() {
    roundtrip("(program 1.0.0 (con integer 999999999999999999999999999999))");
}

#[test]
fn flat_roundtrip_bool_true() {
    roundtrip("(program 1.0.0 (con bool True))");
}

#[test]
fn flat_roundtrip_bool_false() {
    roundtrip("(program 1.0.0 (con bool False))");
}

#[test]
fn flat_roundtrip_unit() {
    roundtrip("(program 1.0.0 (con unit ()))");
}

#[test]
fn flat_roundtrip_bytestring_empty() {
    roundtrip("(program 1.0.0 (con bytestring #))");
}

#[test]
fn flat_roundtrip_bytestring() {
    roundtrip("(program 1.0.0 (con bytestring #deadbeef))");
}

#[test]
fn flat_roundtrip_bytestring_long() {
    // 64 bytes - will span multiple chunks in FLAT encoding
    roundtrip("(program 1.0.0 (con bytestring #deadbeefcafebabe0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f))");
}

#[test]
fn flat_roundtrip_string_empty() {
    roundtrip(r#"(program 1.0.0 (con string ""))"#);
}

#[test]
fn flat_roundtrip_string() {
    roundtrip(r#"(program 1.0.0 (con string "hello world"))"#);
}

#[test]
fn flat_roundtrip_string_unicode() {
    roundtrip(r#"(program 1.0.0 (con string "héllo wörld 🌍"))"#);
}

#[test]
fn flat_roundtrip_string_long() {
    // String longer than 255 bytes to test chunked encoding
    let long_str = "a]b[c".repeat(100);
    let source = format!(r#"(program 1.0.0 (con string "{long_str}"))"#);
    roundtrip(&source);
}

// --- Lambda/application ---

#[test]
fn flat_roundtrip_identity() {
    roundtrip("(program 1.0.0 (lam x x))");
}

#[test]
fn flat_roundtrip_apply() {
    roundtrip("(program 1.0.0 [(lam x x) (con integer 1)])");
}

#[test]
fn flat_roundtrip_nested_lambda() {
    roundtrip("(program 1.0.0 (lam x (lam y [y x])))");
}

// --- Force/Delay ---

#[test]
fn flat_roundtrip_delay_force() {
    roundtrip("(program 1.0.0 (force (delay (con integer 1))))");
}

// --- Builtins ---

#[test]
fn flat_roundtrip_builtin_add() {
    roundtrip("(program 1.0.0 (builtin addInteger))");
}

#[test]
fn flat_roundtrip_builtin_applied() {
    roundtrip_eval("(program 1.0.0 [(builtin addInteger) (con integer 1) (con integer 2)])");
}

#[test]
fn flat_roundtrip_builtin_ifThenElse() {
    roundtrip_eval(
        "(program 1.0.0 (force [(force (builtin ifThenElse)) (con bool True) (delay (con integer 1)) (delay (con integer 2))]))",
    );
}

// --- Constr/Case (v1.1.0) ---

#[test]
fn flat_roundtrip_constr() {
    roundtrip("(program 1.1.0 (constr 0 (con integer 1) (con integer 2)))");
}

#[test]
fn flat_roundtrip_case() {
    roundtrip_eval(
        "(program 1.1.0 (case (constr 1 (con integer 42)) (lam x (con integer 0)) (lam x x)))",
    );
}

// --- Error ---

#[test]
fn flat_roundtrip_error() {
    roundtrip("(program 1.0.0 (error))");
}

// --- Complex programs ---

#[test]
fn flat_roundtrip_trace_with_string() {
    roundtrip_eval(
        r#"(program 1.0.0 [(force (builtin trace)) (con string "hello from trace") (con unit ())])"#,
    );
}

#[test]
fn flat_roundtrip_equals_string() {
    roundtrip_eval(
        r#"(program 1.0.0 [(builtin equalsString) (con string "abc") (con string "abc")])"#,
    );
}

#[test]
fn flat_roundtrip_append_string() {
    roundtrip_eval(
        r#"(program 1.0.0 [(builtin appendString) (con string "hello ") (con string "world")])"#,
    );
}

#[test]
fn flat_roundtrip_encode_utf8() {
    roundtrip_eval(
        r#"(program 1.0.0 [(builtin encodeUtf8) (con string "hello")])"#,
    );
}

#[test]
fn flat_roundtrip_decode_utf8() {
    roundtrip_eval(
        r#"(program 1.0.0 [(builtin decodeUtf8) (con bytestring #68656c6c6f)])"#,
    );
}

#[test]
fn flat_roundtrip_sha2() {
    roundtrip_eval(
        r#"(program 1.0.0 [(builtin sha2_256) (con bytestring #deadbeef)])"#,
    );
}

// --- Benchmark script round-trip (binary files) ---

#[test]
fn flat_decode_benchmark_scripts() {
    let data_dir = std::path::Path::new("benches/use_cases/plutus_use_cases");
    if !data_dir.exists() {
        return; // Skip if benchmark data not available
    }

    for entry in std::fs::read_dir(data_dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let script = std::fs::read(&path).unwrap();

        let arena = Arena::new();
        let program = flat::decode::<DeBruijn>(&arena, &script)
            .unwrap_or_else(|e| panic!("Failed to decode {file_name}: {e:?}"));

        // We don't care if eval succeeds (some scripts need args),
        // we just care that decode succeeded
        let _ = program.eval(&arena);
    }
}

/// Decode, eval, re-encode, re-decode and verify for benchmark scripts
#[test]
fn flat_roundtrip_benchmark_scripts() {
    let data_dir = std::path::Path::new("benches/use_cases/plutus_use_cases");
    if !data_dir.exists() {
        return;
    }

    for entry in std::fs::read_dir(data_dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let script = std::fs::read(&path).unwrap();

        let arena = Arena::new();
        let program = match flat::decode::<DeBruijn>(&arena, &script) {
            Ok(p) => p,
            Err(_) => continue, // Skip scripts that fail to decode
        };

        // Re-encode
        let re_encoded = flat::encode(program)
            .unwrap_or_else(|e| panic!("Failed to re-encode {file_name}: {e:?}"));

        // Re-decode
        let arena2 = Arena::new();
        let re_decoded = flat::decode::<DeBruijn>(&arena2, &re_encoded)
            .unwrap_or_else(|e| panic!("Failed to re-decode {file_name}: {e:?}"));

        assert_eq!(
            program.term, re_decoded.term,
            "Round-trip mismatch for {file_name}"
        );
    }
}
