use bumpalo::Bump;
use divan::{AllocProfiler, Bencher};
use itertools::Itertools;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use uplc_turbo::{
    arena::Arena,
    binder::DeBruijn,
    flat,
    machine::{ExBudget, PlutusVersion},
};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

const TURBO_DATA_DIR: &str = "benches/turbo/samples";
const TURBO_ARCHIVE_URL: &str = "https://pub-2239d82d9a074482b2eb2c886191cb4e.r2.dev/turbo.tar.xz";

// Initial capacity of the arena, in bytes.
const BUMP_ARENA_CAPACITY: usize = 10485760; // 10 MB

// All known samples, loaded lazily
static SAMPLES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| samples());

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

fn samples() -> Vec<PathBuf> {
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

    // Collect all files from all subdirectories.
    entries
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .flat_map(|dir| {
            fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>()
        })
        .sorted()
        .collect()
}

fn collect_scripts(files: &[PathBuf]) -> Vec<(Vec<u8>, PlutusVersion)> {
    files
        .iter()
        .map(|path| {
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();

            let plutus_version = detect_plutus_version(file_stem);

            let cbor = std::fs::read(path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

            let flat = minicbor::decode::<CborWrapped>(&cbor)
                .unwrap_or_else(|e| panic!("Failed to decode CBOR from {}: {}", path.display(), e))
                .0;

            (flat, plutus_version)
        })
        .collect::<Vec<_>>()
}

fn bench_turbo(arena: &mut Arena) -> impl FnMut((Vec<u8>, PlutusVersion)) + use<'_> {
    move |(flat, plutus_version)| {
        let program = flat::decode::<DeBruijn>(arena, &flat).expect("Failed to decode");

        let result = program.eval_version_budget(arena, plutus_version, ExBudget::max());

        let _term = result.term.expect("Failed to evaluate");

        arena.reset();
    }
}

#[divan::bench(sample_count = SAMPLES.len() as u32)]
fn turbo(bencher: Bencher) {
    let mut arena = Arena::from_bump(Bump::with_capacity(BUMP_ARENA_CAPACITY));
    let mut scripts = collect_scripts(&SAMPLES);
    bencher
        .with_inputs(|| scripts.pop().unwrap())
        .bench_local_values(bench_turbo(&mut arena));
}

fn main() {
    divan::main();
}
