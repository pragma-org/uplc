use bumpalo::Bump;
use criterion::{criterion_group, Criterion};
use itertools::Itertools;
use std::{fs, time::Duration};
use uplc_turbo::{arena::Arena, binder::DeBruijn, flat};

pub fn bench_plutus_use_cases(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/benchmarks/plutus_use_cases");

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

            let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));

            c.bench_function(&file_name, |b| {
                b.iter(|| {
                    let program =
                        flat::decode::<DeBruijn>(&arena, &script).expect("Failed to decode");

                    let result = program.eval(&arena);

                    let _term = result.term.expect("Failed to evaluate");

                    arena.reset();
                })
            });
        }
    }
}

criterion_group! {
    name = plutus_use_cases;
    config = Criterion::default()
    .sample_size(10)
    .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(100));
    targets = bench_plutus_use_cases
}
