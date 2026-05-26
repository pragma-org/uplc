# A Lighting Fast UPLC Evaluator

## Dev

### Conformance tests

The conformance suite runs as two parallel harnesses under `crates/uplc/tests/conformance/`:

- **`textual/`**: the upstream Plutus textual fixtures (`.uplc` + `.uplc.expected` + `.uplc.budget.expected`) verbatim, exercised by `tests/conformance.rs` via the `syn` parser. This is the canonical conformance suite and direct evidence we cover what the reference implementation does, including ~169 cases (BLS programs, parser-only failures) that have no flat counterpart.
- **`flat/`**: `fixture.json` files (hex-encoded input bytes + a tagged `expected` describing success output + budget or a pinned rejection layer), exercised by `tests/conformance_flat.rs`. Derived from the textual suite, this exercises the flat decoding path used on-chain.

Run both with:

```bash
cargo test -p amaru-uplc --tests
```

### Refreshing the textual suite

Install [just](https://github.com/casey/just), then:

```bash
just download-plutus-tests
```

This replaces `crates/uplc/tests/conformance/textual/` with the latest fixtures from [IntersectMBO/plutus](https://github.com/IntersectMBO/plutus)'s `plutus-conformance/test-cases/uplc/evaluation/`. The `flat/` suite is untouched; it isn't guaranteed to track upstream byte-for-byte and has no automated sync.

See `crates/uplc/tests/conformance/flat/README.md` for the flat layout, hand-crafted negatives, and which classes of upstream fixtures live only in `textual/`.
