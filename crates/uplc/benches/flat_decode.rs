use amaru_uplc::{arena::Arena, binder::DeBruijn, flat};
use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{fs, time::Duration};

/// Benchmark FLAT decoding only (no evaluation).
pub fn bench_flat_decode(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/use_cases/plutus_use_cases");

    let mut scripts: Vec<(String, Vec<u8>)> = Vec::new();

    for entry in fs::read_dir(data_dir).unwrap().map(|e| e.unwrap()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".flat", "");
        let bytes = fs::read(&path).unwrap();

        // Only include scripts that decode successfully
        let arena = Arena::new();
        if flat::decode_ungated::<DeBruijn>(&arena, &bytes).is_ok() {
            scripts.push((name, bytes));
        }
    }

    scripts.sort_by(|a, b| a.0.cmp(&b.0));

    let mut group = c.benchmark_group("flat_decode");
    group.measurement_time(Duration::from_secs(5));

    for (name, bytes) in &scripts {
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(BenchmarkId::new("decode", name), bytes, |b, script| {
            let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));
            b.iter(|| {
                let _program = flat::decode_ungated::<DeBruijn>(&arena, script).unwrap();
                arena.reset();
            });
        });
    }

    group.finish();

    // Also bench decode+eval together for comparison
    let mut group2 = c.benchmark_group("flat_decode_eval");
    group2.measurement_time(Duration::from_secs(5));

    for (name, bytes) in &scripts {
        group2.bench_with_input(BenchmarkId::new("decode+eval", name), bytes, |b, script| {
            let mut arena = Arena::from_bump(Bump::with_capacity(1_048_576));
            b.iter(|| {
                let program = flat::decode_ungated::<DeBruijn>(&arena, script).unwrap();
                let result = program.eval(&arena);
                let _ = result.term;
                arena.reset();
            });
        });
    }

    group2.finish();
}

criterion_group! {
    name = flat_decode;
    config = Criterion::default();
    targets = bench_flat_decode
}

criterion_main!(flat_decode);
