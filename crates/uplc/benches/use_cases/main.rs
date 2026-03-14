use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use std::{fs, time::Duration};
use uplc_turbo::{arena::Arena, binder::DeBruijn, flat};

/// Set UPLC_BENCH_MODE=bytecode to benchmark the bytecode VM (AOT compiled).
/// Default is "ast" (the standard AST interpreter).
fn use_bytecode() -> bool {
    std::env::var("UPLC_BENCH_MODE").map_or(false, |v| v == "bytecode")
}

pub fn bench_plutus_use_cases(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/use_cases/plutus_use_cases");
    let bytecode_mode = use_bytecode();

    if bytecode_mode {
        eprintln!(">>> Benchmarking in BYTECODE mode (AOT compiled)");
    } else {
        eprintln!(">>> Benchmarking in AST interpreter mode");
    }

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

            if bytecode_mode {
                bench_bytecode(c, &file_name, &script);
            } else {
                bench_ast(c, &file_name, &script);
            }
        }
    }
}

fn bench_ast(c: &mut Criterion, file_name: &str, script: &[u8]) {
    let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

    c.bench_function(file_name, |b| {
        b.iter(|| {
            let program =
                flat::decode::<DeBruijn>(&arena, script).expect("Failed to decode");

            let result = program.eval(&arena);

            let _term = result.term.expect("Failed to evaluate");

            arena.reset();
        })
    });
}

fn bench_bytecode(c: &mut Criterion, file_name: &str, script: &[u8]) {
    use uplc_turbo::{
        bytecode::{compiler, vm},
        machine::{
            cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3,
            BuiltinSemantics, CostModel, ExBudget,
        },
    };

    // AOT compile once outside the measurement loop
    let compile_arena = Box::leak(Box::new(Arena::new()));
    let program = match flat::decode::<DeBruijn>(compile_arena, script) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("EVAL_FAIL: {}", file_name);
            return;
        }
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
            eprintln!("EVAL_FAIL: {}", file_name);
            return;
        }
    }

    let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

    c.bench_function(file_name, |b| {
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

criterion_group! {
    name = plutus_use_cases;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10));
    targets = bench_plutus_use_cases
}

criterion_main! {
    plutus_use_cases,
}
