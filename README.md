# A Lightning Fast UPLC Evaluator

## Crates

| Crate | Description |
|-------|-------------|
| [`uplc-turbo`](crates/uplc) | Core library — parser, flat codec, CEK machine, cost model |
| [`pluton`](crates/pluton) | CLI tool for evaluating UPLC programs |
| [`uplc_macros`](crates/uplc_macros) | Proc macros for test generation |

## Usage

### As a library

```rust
use uplc_turbo::{arena::Arena, binder::DeBruijn, flat::decode};

let arena = Arena::new();
let program: &Program<DeBruijn> = decode(&arena, &bytes).unwrap();
let result = program.eval(&arena);
```

### As a CLI

```bash
cargo run -p pluton -- eval -f program.uplc
cargo run -p pluton -- eval -f program.flat --flat
```

## Dev

### Running tests

```bash
cargo test
```

### Updating conformance tests

Install [just](https://github.com/casey/just).

```bash
just download-plutus-tests
```

### Benchmarks

```bash
cargo bench
```

## License

[Apache-2.0](LICENSE)
