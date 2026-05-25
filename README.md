# A Lighting Fast UPLC Evaluator

## Dev

### Conformance tests

`crates/uplc/tests/conformance/flat/` is the conformance suite. Each fixture is a `fixture.json` (hex-encoded input bytes + a tagged `expected` describing success output + budget or a pinned rejection layer), exercised by `tests/conformance.rs`. This tests the `flat::decode_strict` path used on-chain.

Run with:

```bash
cargo test -p amaru-uplc --tests
```

The suite originally derived from [IntersectMBO/plutus](https://github.com/IntersectMBO/plutus)'s textual conformance fixtures (`plutus-conformance/test-cases/uplc/evaluation/`), converted to flat. It isn't guaranteed to track the upstream suite byte-for-byte and there is no automated sync.

See `crates/uplc/tests/conformance/flat/README.md` for the layout, the hand-crafted negatives, and which classes of failure don't translate from textual to flat.
