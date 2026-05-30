use amaru_uplc::{arena::Arena, machine::PlutusVersion, syn::parse_program};

fn run_conformance(file_contents: &str, expected_output: &str, expected_budget: &str) {
    let arena = Arena::new();

    let Ok(program) = parse_program(&arena, file_contents).into_result() else {
        pretty_assertions::assert_eq!("parse error", expected_output);
        pretty_assertions::assert_eq!("parse error", expected_budget);

        return;
    };

    let result = program.eval_version(&arena, PlutusVersion::V3);

    let info = result.info;

    let Ok(term) = result.term else {
        pretty_assertions::assert_eq!("evaluation failure", expected_output);
        pretty_assertions::assert_eq!("evaluation failure", expected_budget);

        return;
    };

    let expected = parse_program(&arena, expected_output)
        .into_result()
        .unwrap();

    pretty_assertions::assert_eq!(expected.term, term);

    let remaining_budget = format!(
        "({{cpu: {}\n| mem: {}}})",
        info.remaining_budget.cpu, info.remaining_budget.mem
    );

    pretty_assertions::assert_eq!(remaining_budget, expected_budget);
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
