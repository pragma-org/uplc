use amaru_uplc::{
    arena::Arena,
    binder::DeBruijn,
    bytecode::{compiler, vm},
    flat,
    machine::{
        cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3, BuiltinSemantics, CostModel,
        ExBudget,
    },
};
use bumpalo::Bump;

fn main() {
    let path = std::env::args().nth(1).expect("provide a .flat file");
    let iters: usize = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "1000".to_string())
        .parse()
        .unwrap();
    let mode = std::env::args().nth(3).unwrap_or_else(|| "bc".to_string());

    let script = std::fs::read(&path).unwrap();

    if mode == "bc" {
        let compile_arena = Box::leak(Box::new(Arena::new()));
        let program =
            flat::decode_ungated::<DeBruijn>(compile_arena, &script).expect("decode failed");
        let compiled = Box::leak(Box::new(compiler::compile(
            (
                program.version.major(),
                program.version.minor(),
                program.version.patch(),
            ),
            program.term,
        )));

        let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));
        for _ in 0..iters {
            let result = vm::execute(
                &arena,
                compiled,
                ExBudget::default(),
                CostModel::<BuiltinCostsV3>::default(),
                BuiltinSemantics::V2,
            );
            let _ = result.term.expect("eval failed");
            arena.reset();
        }
    } else {
        let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));
        for _ in 0..iters {
            let program = flat::decode_ungated::<DeBruijn>(&arena, &script).expect("decode failed");
            let result = program.eval(&arena);
            let _ = result.term.expect("eval failed");
            arena.reset();
        }
    }
}
