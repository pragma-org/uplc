use bumpalo::Bump;
use criterion::{criterion_group, Criterion};
use itertools::Itertools;
use std::{fs, time::Duration};
use uplc_turbo::{binder::DeBruijn, flat, machine::PlutusVersion};

#[derive(Debug)]
struct CborWrappped(Vec<u8>);

impl<'d, C> minicbor::Decode<'d, C> for CborWrappped {
    fn decode(
        d: &mut minicbor::Decoder<'d>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;
        Ok(CborWrappped(bytes.to_vec()))
    }
}

pub fn bench_turbo(c: &mut Criterion) {
    let data_dir = std::path::Path::new("benches/benchmarks/turbo");

    let dir = fs::read_dir(data_dir).unwrap();

    if dir.count() == 0 {
        panic!(
            "missing turbo benchmarks; download archive at {} and unpack it under {}",
            "https://pub-2239d82d9a074482b2eb2c886191cb4e.r2.dev/turbo.tar.xz",
            data_dir.to_str().unwrap_or_default(),
        );
    }

    let dir = fs::read_dir(data_dir).unwrap();

    for subdir in dir
        .map(|entry| entry.unwrap())
        .map(|entry| entry.path())
        .sorted()
    {
        if subdir.is_dir() {
            for entry in fs::read_dir(subdir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let file_name = format!(
                    "turbo~{}",
                    path.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace(".flat", "")
                );

                let plutus_version = if file_name.ends_with("v1") {
                    PlutusVersion::V1
                } else if file_name.ends_with("v2") {
                    PlutusVersion::V2
                } else if file_name.ends_with("v3") {
                    PlutusVersion::V3
                } else {
                    panic!("cannot decode plutus version from filename: {file_name:?}");
                };

                let cbor = std::fs::read(&path).unwrap();
                let mut arena = Bump::with_capacity(1_048_576);
                let CborWrappped(flat) = minicbor::decode(&cbor).expect("cannot decode from CBOR");

                c.bench_function(&file_name, |b| {
                    b.iter(|| {
                        let program =
                            flat::decode::<DeBruijn>(&arena, &flat).expect("Failed to decode");

                        let result = program.eval_version(&arena, plutus_version);

                        let _term = result.term.expect("Failed to evaluate");

                        arena.reset();
                    })
                });
            }
        }
    }
}

criterion_group! {
    name = turbo;
    config = Criterion::default()
            .measurement_time(Duration::from_secs(10));
    targets = bench_turbo
}

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

            let mut arena = Bump::with_capacity(1_048_576);

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
        .measurement_time(Duration::from_secs(10));
    targets = bench_plutus_use_cases
}
