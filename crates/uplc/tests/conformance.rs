use uplc_macros::generate_tests;
use uplc_turbo::machine::PlutusVersion;

fn run_test(file_contents: &str, expected_output: &str, expected_budget: &str) {
    //let arena = bumpalo::Bump::new();
    let arena = bumpalo::Bump::with_capacity(1024_048_576);

    println!("Begin Parsing");
    let Ok(program) = uplc_turbo::syn::parse_program(&arena, file_contents).into_result() else {
        pretty_assertions::assert_eq!("parse error", expected_output);
        pretty_assertions::assert_eq!("parse error", expected_budget);

        return;
    };
println!("Begin Evaluation");
    let result = program.eval_version(&arena, PlutusVersion::V3);
println!("Evaluation complete");
    let info = &result.info;
//    println!("Info obtained: {:?}", result.term);

    let Ok(term) = result.term else {
        pretty_assertions::assert_eq!("evaluation failure", expected_output);
        pretty_assertions::assert_eq!("evaluation failure", expected_budget);

        return;
    };

    let expected = uplc_turbo::syn::parse_program(&arena, expected_output)
        .into_result()
        .unwrap();
//    println!("expected: {:?}", expected.term);
//        println!("res: {:?}", term);
        
    pretty_assertions::assert_eq!(expected.term, term);

    let consumed_budget = format!(
        "({{cpu: {}\r\n| mem: {}}})",
        info.consumed_budget.cpu, info.consumed_budget.mem
    );

//        println!("consumed budget: {:?}", consumed_budget);
//        println!("expected budget: {:?}",  expected_budget);

    pretty_assertions::assert_eq!(consumed_budget, expected_budget);
}

generate_tests!("conformance");
