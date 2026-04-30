use bumpalo::Bump;
use uplc_turbo::{
    arena::Arena,
    binder::DeBruijn,
    bytecode::{compiler, vm},
    flat,
    machine::{
        cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3, BuiltinSemantics, CostModel,
        ExBudget,
    },
};

fn main() {
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "benches/use_cases/plutus_use_cases/auction_1-2.flat".to_string());

    let script = std::fs::read(&script_path).expect("Failed to read script");

    // Compile once (AOT)
    let compile_arena = Box::leak(Box::new(Arena::new()));
    let program = flat::decode::<DeBruijn>(compile_arena, &script).expect("Failed to decode");
    let compiled = Box::leak(Box::new(compiler::compile(
        (
            program.version.major(),
            program.version.minor(),
            program.version.patch(),
        ),
        program.term,
    )));

    let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Warmup
    for _ in 0..5 {
        let result = vm::execute(
            &arena,
            compiled,
            ExBudget::default(),
            CostModel::<BuiltinCostsV3>::default(),
            BuiltinSemantics::V2,
        );
        let _term = result.term.expect("Failed to evaluate");
        arena.reset();
    }

    // Profile
    for _ in 0..iterations {
        let result = vm::execute(
            &arena,
            compiled,
            ExBudget::default(),
            CostModel::<BuiltinCostsV3>::default(),
            BuiltinSemantics::V2,
        );
        let _term = result.term.expect("Failed to evaluate");
        arena.reset();
    }
}
