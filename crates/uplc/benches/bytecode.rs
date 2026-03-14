use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use std::{fs, time::Duration};
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

/// Benchmark: pre-compiled bytecode execution only (AOT).
/// Compilation happens once outside the measurement loop.
pub fn bench_bytecode_aot(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/use_cases/plutus_use_cases");

    for path in fs::read_dir(data_dir)
        .unwrap()
        .map(|entry| entry.unwrap())
        .map(|entry| entry.path())
        .sorted()
    {
        if path.is_file() {
            let file_name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".flat", "");

            let script = std::fs::read(&path).unwrap();

            // Pre-compile once: decode FLAT → AST → bytecode
            let compile_arena = Box::leak(Box::new(Arena::new()));
            let program = match flat::decode::<DeBruijn>(compile_arena, &script) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let compiled = Box::leak(Box::new(compiler::compile(
                (program.version.major(), program.version.minor(), program.version.patch()),
                program.term,
            )));

            // Verify it works
            {
                let test_arena = Arena::new();
                let result = vm::execute(
                    &test_arena,
                    compiled,
                    ExBudget::default(),
                    CostModel::<BuiltinCostsV3>::default(),
                    BuiltinSemantics::V2,
                );
                if result.term.is_err() {
                    continue;
                }
            }

            let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

            c.bench_function(&format!("bc_{file_name}"), |b| {
                b.iter(|| {
                    let result = vm::execute(
                        &arena,
                        compiled,
                        ExBudget::default(),
                        CostModel::<BuiltinCostsV3>::default(),
                        BuiltinSemantics::V2,
                    );

                    let _term = result.term.expect("Failed to evaluate");

                    arena.reset();
                })
            });
        }
    }
}

criterion_group! {
    name = bytecode_bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10));
    targets = bench_bytecode_aot
}

criterion_main! {
    bytecode_bench,
}
