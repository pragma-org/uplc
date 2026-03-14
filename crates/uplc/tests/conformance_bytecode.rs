use uplc_macros::generate_tests;
use uplc_turbo::machine::PlutusVersion;

fn run_test(file_contents: &str, expected_output: &str, expected_budget: &str) {
    let arena = uplc_turbo::arena::Arena::new();

    let Ok(program) = uplc_turbo::syn::parse_program(&arena, file_contents).into_result() else {
        pretty_assertions::assert_eq!("parse error", expected_output);
        pretty_assertions::assert_eq!("parse error", expected_budget);

        return;
    };

    // Compile to bytecode
    let compiled = uplc_turbo::bytecode::compiler::compile(
        (
            program.version.major(),
            program.version.minor(),
            program.version.patch(),
        ),
        program.term,
    );

    // Execute via bytecode VM
    let result = uplc_turbo::bytecode::vm::execute(
        &arena,
        &compiled,
        uplc_turbo::machine::ExBudget::default(),
        uplc_turbo::machine::CostModel::<
            uplc_turbo::machine::cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3,
        >::default(),
        uplc_turbo::machine::BuiltinSemantics::V2,
    );

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
