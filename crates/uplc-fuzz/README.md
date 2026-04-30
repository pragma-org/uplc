# uplc-fuzz

Differential fuzzer for UPLC (Untyped Plutus Language Core) virtual machines. Generates random programs and compares the CEK tree-walking interpreter against the bytecode VM to detect divergences in evaluation results and budget consumption.

## Building

```bash
cargo build --bin uplc-fuzz --release
```

## Usage

```bash
# Run with defaults (all cores, Plutus V3, TUI dashboard, unlimited duration)
cargo run --bin uplc-fuzz

# Run for 10 minutes with 4 workers
cargo run --bin uplc-fuzz -j 4 -d 600

# Plain output (no TUI, suitable for CI)
cargo run --bin uplc-fuzz --no-tui -d 300

# Reproducible run with fixed seed
cargo run --bin uplc-fuzz --seed 42 -d 3600

# Fuzz Plutus V1 with tight budgets
cargo run --bin uplc-fuzz --version v1 --budget-cpu 1000000 --budget-mem 1000000
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `-j, --jobs` | all CPUs | Number of worker threads |
| `-d, --duration` | 0 (unlimited) | Run duration in seconds |
| `-o, --output` | `fuzz-output` | Output directory for divergence catalog |
| `--seed` | random | RNG seed for reproducibility |
| `--version` | `v3` | Plutus version: `v1`, `v2`, `v3` |
| `--budget-cpu` | 10000000000 | CPU budget limit |
| `--budget-mem` | 14000000 | Memory budget limit |
| `--batch-size` | 64 | Programs generated per batch |
| `--no-tui` | off | Disable TUI, use plain stderr output |
| `--replay` | — | Replay a single divergence file |
| `--debug` | off | Print debug info during replay (top-level term variants) |

## Output

Divergences are saved to the output directory:

- `catalog.jsonl` — one JSON record per divergence (outcomes, budgets, program seed)
- `divergence_NNNN.uplc` — standalone UPLC source with metadata comments

Example `.uplc` file:

```
-- Divergence: ResultMismatch
-- CEK outcome: Success(...)
-- CEK budget: cpu=123456 mem=7890
-- BC outcome: EvaluationFailure("...")
-- BC budget: cpu=0 mem=0
(program 1.1.0 ...)
```

## Replaying divergences

```bash
# Replay a saved divergence
cargo run --bin uplc-fuzz --replay fuzz-output/divergence_0001.uplc

# With debug output (prints top-level term variant)
cargo run -bin uplc-fuzz --replay fuzz-output/divergence_0001.uplc --debug
```

Replay exits with code 1 on divergence, making it usable in scripts:

```bash
for f in fuzz-output/divergence_*.uplc; do
    cargo run -bin uplc-fuzz --replay "$f" || echo "DIVERGING: $f"
done
```

## Architecture

### Engines

Both engines evaluate the same program and their results are compared:

- **CEK machine** — tree-walking interpreter (`program.eval_version_budget`), the reference implementation
- **Bytecode VM** — compiles to bytecode then executes (`bytecode::compile` + `bytecode::vm::execute`)

### Program generators

Two weighted generation strategies:

| Generator | Weight | Description |
|-----------|--------|-------------|
| **RandomStructural** | 2 | Pure random AST generation (depth up to 12). High volume, broad coverage. |
| **BuiltinAware** | 4 | Generates well-typed builtin applications with correct force counts and plausible argument types. Higher success rate. |

### Divergence kinds

| Kind | Meaning |
|------|---------|
| `ResultMismatch` | Different evaluation outcomes (success vs failure, or different values) |
| `BudgetMismatch` | Same outcome, different CPU/memory consumption |
| `ResultAndBudgetMismatch` | Both outcome and budget differ |
| `Panic` | One engine panicked (crash bug) |

Two evaluation failures with different error messages are **not** considered a divergence — only the outcome category matters.

### Worker loop

Each worker thread:

1. Selects a generator by weight
2. Generates a batch of program seeds
3. Evaluates each with both CEK and bytecode (wrapped in `catch_unwind`)
4. Compares results via `compare_results()`
5. Sends divergences and stats to the coordinator over a channel
6. Periodically resets its arena to bound memory usage

### TUI dashboard

When running with the TUI (default), the dashboard shows:

- Throughput sparkline (programs/sec over time)
- Total programs tested, success/error/divergence counts
- Divergence breakdown by kind with percentage bars
- Recent divergence log
