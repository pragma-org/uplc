use uplc_macros::generate_tests;

fn run_test(file_contents: &str, expected_output: &str, expected_budget: &str) {
    let arena = amaru_uplc::arena::Arena::new();

    let Ok(program) = amaru_uplc::syn::parse_program(&arena, file_contents).into_result() else {
        pretty_assertions::assert_eq!("parse error", expected_output);
        pretty_assertions::assert_eq!("parse error", expected_budget);

        return;
    };

    // Compile to bytecode
    let compiled = amaru_uplc::bytecode::compiler::compile(
        (
            program.version.major(),
            program.version.minor(),
            program.version.patch(),
        ),
        program.term,
    );

    // Execute via bytecode VM
    let result = amaru_uplc::bytecode::vm::execute(
        &arena,
        &compiled,
        amaru_uplc::machine::ExBudget::default(),
        amaru_uplc::machine::CostModel::<
            amaru_uplc::machine::cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3,
        >::default(),
        amaru_uplc::machine::BuiltinSemantics::V2,
    );

    let info = result.info;

    let Ok(term) = result.term else {
        pretty_assertions::assert_eq!("evaluation failure", expected_output);
        pretty_assertions::assert_eq!("evaluation failure", expected_budget);

        return;
    };

    let expected = amaru_uplc::syn::parse_program(&arena, expected_output)
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
