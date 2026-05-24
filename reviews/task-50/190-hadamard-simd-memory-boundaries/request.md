# Review Request: Hadamard SIMD Memory Boundaries

## Summary

Consolidated split SIMD memory-access unsafe blocks in `src/quant/hadamard.rs`.

- AVX2 stage-width processing now uses one bounded unsafe block for the vector branch and one for the scalar tail branch, instead of separate pointer-add and load/store blocks.
- NEON stage processing now keeps each four-lane load/compute/store group inside one unsafe block.

This is RaBitQ-adjacent cleanup because Hadamard is the transform path used by the quantization/search stack. The change preserves the existing power-of-two, width-divisibility, and target-feature contracts; it only tightens where the memory-access proof is stated.

## Unsafe Ledger

- `src/quant/hadamard.rs`: `41 -> 37`
- `src/`: `2664 -> 2660`

## Validation

- `rustfmt --check src/quant/hadamard.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib quant::hadamard --no-default-features --features pg18,bench --no-run`
- Runtime Hadamard unit test attempt is blocked before test execution by the existing `undefined symbol: LockBuffer` loader failure.

Artifact logs and command metadata are in `artifacts/manifest.md`.
