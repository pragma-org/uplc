use uplc_turbo::{
    arena::Arena,
    binder::DeBruijn,
    bytecode::{compiler, vm},
    flat,
    machine::{
        cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3, BuiltinSemantics, CostModel,
        ExBudget, PlutusVersion,
    },
};

fn main() {
    let path = std::env::args().nth(1).expect("provide a .flat file");
    let script = std::fs::read(&path).unwrap();

    // AST eval
    let arena = Arena::new();
    let program = flat::decode::<DeBruijn>(&arena, &script).expect("decode failed");
    let ast_result = program.eval_version(&arena, PlutusVersion::V3);
    let ast_budget = ast_result.info.consumed_budget;
    match &ast_result.term {
        Ok(term) => eprintln!("AST OK: {term:?}"),
        Err(e) => eprintln!("AST FAIL: {e:?}"),
    }
    eprintln!("AST budget: cpu={}, mem={}", ast_budget.cpu, ast_budget.mem);

    // Bytecode eval
    let arena2 = Arena::new();
    let program2 = flat::decode::<DeBruijn>(&arena2, &script).expect("decode failed");
    let compiled = compiler::compile(
        (
            program2.version.major(),
            program2.version.minor(),
            program2.version.patch(),
        ),
        program2.term,
    );

    let arena3 = Arena::new();
    let result = vm::execute(
        &arena3,
        &compiled,
        ExBudget::default(),
        CostModel::<BuiltinCostsV3>::default(),
        BuiltinSemantics::V2,
    );

    let bc_budget = result.info.consumed_budget;
    match &result.term {
        Ok(term) => eprintln!("BC  OK: {term:?}"),
        Err(e) => eprintln!("BC  FAIL: {e:?}"),
    }
    eprintln!("BC  budget: cpu={}, mem={}", bc_budget.cpu, bc_budget.mem);

    // Compare step counts
    eprintln!("\nAST steps: {:?}", ast_result.info.consumed_budget);
    eprintln!("BC  steps: {:?}", result.info.consumed_budget);

    // If AST passes but BC fails, try running AST with the same budget BC consumed
    // to see roughly where the divergence happens
    if ast_result.term.is_ok() && result.term.is_err() {
        let arena4 = Arena::new();
        let program4 = flat::decode::<DeBruijn>(&arena4, &script).expect("decode failed");
        // Run with limited budget matching what BC used
        let limited = program4.eval_version_budget(
            &arena4,
            PlutusVersion::V3,
            ExBudget {
                cpu: bc_budget.cpu as i64,
                mem: bc_budget.mem as i64,
            },
        );
        match &limited.term {
            Ok(term) => eprintln!("AST with BC budget: OK: {term:?}"),
            Err(e) => eprintln!("AST with BC budget: FAIL: {e:?} (expected — proves divergence is in logic, not budget)"),
        }
    }
}
