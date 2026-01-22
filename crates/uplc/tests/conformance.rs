use uplc_macros::generate_tests;
use uplc_turbo::machine::PlutusVersion;

fn run_test(file_contents: &str, expected_output: &str, expected_budget: &str) {
    let arena = uplc_turbo::arena::Arena::new();

    let Ok(program) = uplc_turbo::syn::parse_program(&arena, file_contents).into_result() else {
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

    let expected = uplc_turbo::syn::parse_program(&arena, expected_output)
        .into_result()
        .unwrap();

    pretty_assertions::assert_eq!(expected.term, term);

    let consumed_budget = format!(
        "({{cpu: {}\n| mem: {}}})",
        info.consumed_budget.cpu, info.consumed_budget.mem
    );

    pretty_assertions::assert_eq!(consumed_budget, expected_budget);
}

generate_tests!("conformance");
