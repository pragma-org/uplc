use bumpalo::Bump;
use criterion::{criterion_group, BatchSize, Criterion};
use itertools::Itertools;
use std::{fs, time::Duration};
use uplc_turbo::{arena::Arena, binder::DeBruijn, flat, machine::PlutusVersion};

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

fn bench_turbo_with_filter<F>(c: &mut Criterion, filter: F, part: &str)
where
    F: Fn(&str) -> bool,
{
    let data_dir = std::path::Path::new("benches/benchmarks/turbo");

    // Check directory exists and has content in one pass
    let entries: Vec<_> = fs::read_dir(data_dir).unwrap().collect();

    if entries.len() <= 1 {
        panic!(
            "missing turbo benchmarks; download archive at {} and unpack it under {}",
            "https://pub-2239d82d9a074482b2eb2c886191cb4e.r2.dev/turbo.tar.xz",
            data_dir.to_str().unwrap_or_default(),
        );
    }

    let subdirs: Vec<_> = entries
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| filter(name))
        })
        .sorted()
        .collect();

    for subdir in subdirs {
        let entries = match fs::read_dir(&subdir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();

            let file_name = format!("turbo~{}~{}", part, file_stem);

            let plutus_version = if file_name.ends_with("v1") {
                PlutusVersion::V1
            } else if file_name.ends_with("v2") {
                PlutusVersion::V2
            } else if file_name.ends_with("v3") {
                PlutusVersion::V3
            } else {
                panic!("cannot decode plutus version from filename: {file_name}");
            };

            let cbor = match std::fs::read(&path) {
                Ok(cbor) => cbor,
                Err(e) => {
                    panic!("Failed to read {}: {}", path.display(), e);
                }
            };

            let flat = match minicbor::decode::<CborWrappped>(&cbor) {
                Ok(CborWrappped(flat)) => flat,
                Err(e) => {
                    panic!("Failed to decode CBOR from {}: {}", path.display(), e);
                }
            };

            c.bench_function(&file_name, |b| {
                b.iter_batched(
                    || Arena::from_bump(Bump::with_capacity(1_048_576)),
                    |arena| {
                        let program =
                            flat::decode::<DeBruijn>(&arena, &flat).expect("Failed to decode");

                        let result = program.eval_version(&arena, plutus_version);

                        let _term = result.term.expect("Failed to evaluate");
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }
}

pub fn bench_turbo_part_1(c: &mut Criterion) {
    bench_turbo_with_filter(
        c,
        |folder| {
            folder
                .parse::<u32>()
                .map_or(false, |num| (290..=395).contains(&num))
        },
        "part1",
    );
}

pub fn bench_turbo_part_2(c: &mut Criterion) {
    bench_turbo_with_filter(
        c,
        |folder| {
            folder
                .parse::<u32>()
                .map_or(false, |num| (396..=452).contains(&num))
        },
        "part2",
    );
}

pub fn bench_turbo_part_3(c: &mut Criterion) {
    bench_turbo_with_filter(
        c,
        |folder| {
            folder
                .parse::<u32>()
                .map_or(false, |num| (453..=499).contains(&num))
        },
        "part3",
    );
}

pub fn bench_turbo_part_4(c: &mut Criterion) {
    bench_turbo_with_filter(
        c,
        |folder| folder.parse::<u32>().map_or(false, |num| num >= 500),
        "part4",
    );
}

criterion_group! {
    name = turbo;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(10))
        .measurement_time(Duration::from_millis(100));
    targets = bench_turbo_part_1, bench_turbo_part_2, bench_turbo_part_3, bench_turbo_part_4
}
