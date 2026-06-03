use amaru_uplc::{
    arena::Arena,
    machine::{default_v3_cost_model, ExBudget, PlutusVersion},
    syn::parse_program,
};

const PLUTUS_VERSION: PlutusVersion = PlutusVersion::V3;
const PROTOCOL_VERSION: (u64, u64) = (11, 0);

fn run_conformance(file_contents: &str, expected_output: &str, expected_budget: &str) {
    let arena = Arena::new();
    let costs = default_v3_cost_model();

    let Ok(program) = parse_program(&arena, file_contents, PROTOCOL_VERSION.0 as u32).into_result()
    else {
        pretty_assertions::assert_eq!("parse error", expected_output);
        pretty_assertions::assert_eq!("parse error", expected_budget);

        return;
    };

    let result = program.eval_with_params(
        &arena,
        PLUTUS_VERSION,
        PROTOCOL_VERSION,
        &costs,
        ExBudget::default(),
    );

    let info = result.info;

    let Ok(term) = result.term else {
        pretty_assertions::assert_eq!("evaluation failure", expected_output);
        pretty_assertions::assert_eq!("evaluation failure", expected_budget);

        return;
    };

    let expected = parse_program(&arena, expected_output, PROTOCOL_VERSION.0 as u32)
        .into_result()
        .unwrap();

    pretty_assertions::assert_eq!(expected.term, term);

    let consumed_budget = format!(
        "({{cpu: {}\n| mem: {}}})",
        info.consumed_budget.cpu, info.consumed_budget.mem
    );

    pretty_assertions::assert_eq!(consumed_budget, expected_budget);
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
