# amaru-uplc

[![Crates.io](https://img.shields.io/crates/v/amaru-uplc.svg)](https://crates.io/crates/amaru-uplc)
[![docs.rs](https://img.shields.io/docsrs/amaru-uplc)](https://docs.rs/amaru-uplc)
[![License](https://img.shields.io/crates/l/amaru-uplc.svg)](LICENSE)

A lightning-fast [UPLC](https://plutus.readthedocs.io/en/latest/reference/uplc-introduction.html)
(Untyped Plutus Language Core) evaluator implemented as a
[CEK machine](https://en.wikipedia.org/wiki/CEK_Machine) in Rust.

UPLC is the low-level bytecode compiled from [Plutus](https://plutus.readthedocs.io/), the
smart-contract language for the [Cardano](https://cardano.org/) blockchain.

## Features

- Full CEK machine evaluation for Plutus V1, V2, and V3
- Arena-allocated term representation for minimal allocator overhead
- Built-in UPLC text-format parser
- Flat and CBOR binary encoding/decoding
- Configurable cost models with `ExBudget` tracking
- Comprehensive built-in functions:
  - Arithmetic and integer operations
  - Byte-string and UTF-8 string operations
  - Cryptographic hashing (SHA-256, SHA-3, Blake2b-256/224, Keccak-256, RIPEMD-160)
  - Signature verification (Ed25519, ECDSA secp256k1, Schnorr secp256k1)
  - BLS12-381 elliptic curve operations (G1, G2, Miller loop, pairing)
  - Bitwise operations (Plutus V3)
  - Plutus data constructors and destructors

## Installation

```toml
[dependencies]
amaru-uplc = "0.4.0"
```

## Usage

### Building and evaluating a program

```rust
use amaru_uplc::{
    arena::Arena,
    binder::DeBruijn,
    program::{Program, Version},
    term::Term,
};

let arena = Arena::new();

// Build a term: addInteger 1 3
let term = Term::add_integer(&arena)
    .apply(&arena, Term::integer_from(&arena, 1))
    .apply(&arena, Term::integer_from(&arena, 3));

let version = Version::plutus_v3(&arena);
let program = Program::<DeBruijn>::new(&arena, version, term);
let result = program.eval(&arena);

assert_eq!(result.term.unwrap(), Term::integer_from(&arena, 4));
```

### Parsing UPLC source text

```rust
use amaru_uplc::{arena::Arena, syn::parse_program};

let arena = Arena::new();
let result = parse_program(&arena, "(program 1.1.0 (addInteger 1 3))");
```

### Selecting Plutus version and budget

```rust
use amaru_uplc::machine::{ExBudget, PlutusVersion};

// Evaluate under Plutus V1 semantics with an unlimited budget
let result = program.eval_version_budget(&arena, PlutusVersion::V1, ExBudget::max());

// Inspect consumed budget and trace logs
println!("CPU: {}", result.info.consumed_budget.cpu);
println!("Mem: {}", result.info.consumed_budget.mem);
for line in &result.info.logs {
    println!("TRACE: {line}");
}
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [just](https://github.com/casey/just) — task runner used for common workflows
- [cargo-nextest](https://nexte.st/) — faster test runner (`cargo install cargo-nextest`)

### Conformance tests

The conformance suite runs as two parallel harnesses under `crates/uplc/tests/conformance/`:

- **`textual/`**: the upstream Plutus textual fixtures (`.uplc` + `.uplc.expected` + `.uplc.budget.expected`) verbatim, exercised by `tests/conformance.rs` via the `syn` parser. This is the canonical conformance suite and direct evidence we cover what the reference implementation does, including ~169 cases (BLS programs, parser-only failures) that have no flat counterpart.
- **`flat/`**: `fixture.json` files (hex-encoded input bytes + a tagged `expected` describing success output + budget or a pinned rejection layer), exercised by `tests/conformance_flat.rs`. Derived from the textual suite, this exercises the flat decoding path used on-chain.

Run both with:

```bash
cargo test -p amaru-uplc --tests
```

### Refreshing the textual suite

The conformance test suite is not vendored. Download it before running tests:

```bash
just download-plutus-tests
```

This replaces `crates/uplc/tests/conformance/textual/` with the latest fixtures from [IntersectMBO/plutus](https://github.com/IntersectMBO/plutus)'s `plutus-conformance/test-cases/uplc/evaluation/`. The `flat/` suite is untouched; it isn't guaranteed to track upstream byte-for-byte and has no automated sync.

See `crates/uplc/tests/conformance/flat/README.md` for the flat layout, hand-crafted negatives, and which classes of upstream fixtures live only in `textual/`.

### Testing

Run the full test suite (unit tests + conformance tests):

```bash
# Using the built-in test harness
cargo test

# Or directly with nextest
cargo nextest run


```

Run only the unit tests (skip conformance):

```bash
cargo nextest run --lib
```

Run only the conformance tests:

```bash
cargo nextest run --test conformance
```

Run a specific test by name:

```bash
cargo nextest run add_integer
cargo nextest run fibonacci
```

Run tests matching a pattern:

```bash
cargo nextest run encode_cbor
```

Run tests in release mode:

```bash
cargo nextest run --release
```

List all available tests without running them:

```bash
cargo test -- --list
```

### Benchmarks

Run all benchmarks:

```bash
cargo bench
```

Run a specific benchmark suite:

```bash
# Microbenchmarks (addInteger, fibonacci)
cargo bench --bench simple

# Real-world Plutus use-case scripts
cargo bench --bench use_cases

# High-throughput bulk evaluation over a large script corpus
cargo bench --bench turbo
```

### Documentation

Build and open the API docs locally:

```bash
cargo doc -p amaru-uplc --open
```

## License

Apache-2.0 — see [LICENSE](LICENSE).
