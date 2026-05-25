use amaru_uplc::{
    arena::Arena,
    binder::DeBruijn,
    flat,
    machine::{ExBudget, PlutusVersion},
    program::Program,
};
use serde::Deserialize;

// Pin to V3 / PV 11 (van Rossem). PV 11 enables all current builtins on V3,
// including the CIP-153 batch (ExpModInteger, DropList, LengthOfArray,
// ListToArray, IndexArray).
const PLUTUS_VERSION: PlutusVersion = PlutusVersion::V3;
const PROTOCOL_VERSION_MAJOR: u32 = 11;

#[derive(Debug, Deserialize)]
struct Fixture {
    input: String,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Expected {
    /// Decode must succeed.
    /// Eval must succeed and produce the given output with the given budget.
    Ok {
        output: String,
        #[serde(deserialize_with = "deserialize_budget")]
        budget: ExBudget,
    },
    /// Decode or eval must fail. `layer` pins which one if set.
    /// if absent, either is acceptable.
    Error {
        #[serde(default)]
        layer: Option<ErrorLayer>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ErrorLayer {
    Decode,
    Eval,
}

fn deserialize_budget<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ExBudget, D::Error> {
    #[derive(Deserialize)]
    struct Raw {
        cpu: i64,
        mem: i64,
    }
    let Raw { cpu, mem } = Raw::deserialize(d)?;
    Ok(ExBudget::new(mem, cpu))
}

fn run_conformance(fixture_json: &str) {
    let fixture: Fixture = serde_json::from_str(fixture_json).expect("parse fixture json");
    let input = hex::decode(&fixture.input).expect("decode input hex");

    let arena = Arena::new();

    let program = match flat::decode_strict::<DeBruijn>(
        &arena,
        &input,
        PLUTUS_VERSION,
        PROTOCOL_VERSION_MAJOR,
    ) {
        Ok(p) => p,
        Err(e) => {
            match &fixture.expected {
                Expected::Ok { .. } => panic!("decode failed but fixture expects success: {e:?}"),
                Expected::Error {
                    layer: Some(ErrorLayer::Eval),
                } => {
                    panic!("fixture pinned `eval` but decode rejected first: {e:?}")
                }
                Expected::Error { .. } => {}
            }
            return;
        }
    };

    if let Expected::Error {
        layer: Some(ErrorLayer::Decode),
    } = &fixture.expected
    {
        panic!("fixture pinned `decode` but decode succeeded; eval will run next");
    }

    let result = program.eval(&arena);

    let term = match result.term {
        Ok(t) => t,
        Err(_) => {
            assert!(
                matches!(fixture.expected, Expected::Error { .. }),
                "evaluation failed but fixture expects success",
            );
            return;
        }
    };

    let Expected::Ok { output, budget } = &fixture.expected else {
        panic!("evaluation succeeded but fixture expects a failure");
    };

    let expected_bytes = hex::decode(output).expect("decode output hex");
    let result_program = Program::new(&arena, program.version, term);
    let encoded = flat::encode(result_program).expect("encoder failed on eval result");

    assert_eq!(hex::encode(&expected_bytes), hex::encode(&encoded));
    assert_eq!(result.info.consumed_budget, *budget);
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
