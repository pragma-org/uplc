use amaru_uplc::{arena::Arena, binder::DeBruijn, flat, machine::ExBudget};
use bumpalo::Bump;

fn main() {
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "benches/use_cases/plutus_use_cases/auction_1-2.flat".to_string());

    let script = std::fs::read(&script_path).expect("Failed to read script");
    let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

    // Warmup
    for _ in 0..5 {
        let program = flat::decode_ungated::<DeBruijn>(&arena, &script).expect("Failed to decode");
        let result = program.eval(&arena);
        let _term = result.term.expect("Failed to evaluate");
        arena.reset();
    }

    // Profile iterations
    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    for _ in 0..iterations {
        let program = flat::decode_ungated::<DeBruijn>(&arena, &script).expect("Failed to decode");
        let result = program.eval(&arena);
        let _term = result.term.expect("Failed to evaluate");
        arena.reset();
    }
}
