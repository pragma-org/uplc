use std::{fs, time::Duration};

use bumpalo::Bump;
use criterion::{criterion_group, Criterion};
use itertools::Itertools;

pub fn run(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/benchmarks/data");

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

            let mut arena = Bump::with_capacity(2_048_000);

            c.bench_function(&file_name, |b| {
                b.iter(|| {
                    let program = uplc::flat::decode(&arena, &script).expect("Failed to decode");

                    let result = program.eval(&arena);

                    let _term = result.term.expect("Failed to evaluate");

                    arena.reset();
                })
            });
        }
    }
}

criterion_group!(haskell, run);
