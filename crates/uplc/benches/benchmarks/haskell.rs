use std::fs;

use bumpalo::Bump;
use criterion::{criterion_group, Criterion};

pub fn run(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/benchmarks/data");

    for entry in fs::read_dir(data_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let script = std::fs::read(&path).unwrap();

            c.bench_function(file_name, |b| {
                b.iter(|| {
                    let arena = Bump::new();

                    let program = uplc::flat::decode(&arena, &script).expect("Failed to decode");

                    let result = program.eval(&arena);

                    let term = result.term.expect("Failed to evaluate");
                })
            });
        }
    }
}

criterion_group!(haskell, run);
