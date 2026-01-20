use bumpalo::Bump;
use criterion::{criterion_group, BatchSize, Criterion};
use itertools::Itertools;
use std::{fs, path::Path, time::Duration};
use uplc_turbo::{arena::Arena, binder::DeBruijn, flat, machine::PlutusVersion};

const TURBO_DATA_DIR: &str = "benches/benchmarks/turbo";
const TURBO_ARCHIVE_URL: &str = "https://pub-2239d82d9a074482b2eb2c886191cb4e.r2.dev/turbo.tar.xz";
const BUMP_ARENA_CAPACITY: usize = 1_048_576;
const NUM_PARTS: usize = 4;

#[derive(Debug)]
struct CborWrapped(Vec<u8>);

impl<'d, C> minicbor::Decode<'d, C> for CborWrapped {
    fn decode(
        d: &mut minicbor::Decoder<'d>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;
        Ok(CborWrapped(bytes.to_vec()))
    }
}

fn detect_plutus_version(file_name: &str) -> PlutusVersion {
    if file_name.ends_with("v1") {
        PlutusVersion::V1
    } else if file_name.ends_with("v2") {
        PlutusVersion::V2
    } else if file_name.ends_with("v3") {
        PlutusVersion::V3
    } else {
        panic!("cannot decode plutus version from filename: {file_name}");
    }
}

fn bench_turbo_with_filter<F>(c: &mut Criterion, filter: F, part: &str)
where
    F: Fn(usize) -> bool,
{
    let data_dir = Path::new(TURBO_DATA_DIR);

    // Check directory exists and has content
    let entries: Vec<_> = fs::read_dir(data_dir)
        .unwrap_or_else(|_| panic!("Failed to read directory: {}", data_dir.display()))
        .collect();

    if entries.len() <= 1 {
        panic!(
            "missing turbo benchmarks; download archive at {} and unpack it under {}",
            TURBO_ARCHIVE_URL,
            data_dir.to_str().unwrap_or_default(),
        );
    }

    // Collect all files from all subdirectories
    let all_files: Vec<_> = entries
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .flat_map(|subdir| {
            fs::read_dir(&subdir)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
        })
        .sorted()
        .collect();

    // Filter files by index for even distribution
    let files: Vec<_> = all_files
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| filter(*idx))
        .map(|(_, path)| path)
        .collect();

    for path in files {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let folder_name = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        let file_name = format!("turbo~{}~{}~{}", part, folder_name, file_stem);

        let plutus_version = detect_plutus_version(&file_name);

        let cbor = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        let flat = minicbor::decode::<CborWrapped>(&cbor)
            .unwrap_or_else(|e| panic!("Failed to decode CBOR from {}: {}", path.display(), e))
            .0;

        c.bench_function(&file_name, |b| {
            b.iter_batched(
                || Arena::from_bump(Bump::with_capacity(BUMP_ARENA_CAPACITY)),
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

macro_rules! bench_turbo_part {
    ($name:ident, $part_num:expr, $part_name:expr) => {
        pub fn $name(c: &mut Criterion) {
            bench_turbo_with_filter(c, |idx| idx % NUM_PARTS == $part_num, $part_name);
        }
    };
}

bench_turbo_part!(bench_turbo_part_1, 0, "part1");
bench_turbo_part!(bench_turbo_part_2, 1, "part2");
bench_turbo_part!(bench_turbo_part_3, 2, "part3");
bench_turbo_part!(bench_turbo_part_4, 3, "part4");

criterion_group! {
    name = turbo;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(10))
        .measurement_time(Duration::from_millis(100));
    targets = bench_turbo_part_1, bench_turbo_part_2, bench_turbo_part_3, bench_turbo_part_4
}
