# Task 50 Review Request: Hadamard Safe SIMD Dispatch

## Summary

Centralized FWHT SIMD target-feature dispatch in `src/quant/hadamard.rs`.

The public dispatch and test helper surfaces now call safe runtime-detected
wrappers:

- `try_fwht_in_place_avx2`
- `try_fwht_in_place_neon`

Those wrappers own the single target-feature unsafe call after runtime feature
detection. The SIMD internals remain unsafe where the target-feature and
intrinsic contracts are irreducible.

## Unsafe Burndown

- `src/quant/hadamard.rs` unsafe grep count: `37 -> 34`
- repository `src` unsafe grep count: `2412 -> 2409`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/quant/hadamard.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib quant::hadamard --no-default-features --features pg18,bench --no-run`
  - Passed.
- Runtime attempt captured for transparency:
  `cargo test --lib quant::hadamard --no-default-features --features pg18,bench`
  failed before running test bodies with a local PostgreSQL symbol lookup error
  (`undefined symbol: LockBuffer`). See
  `artifacts/cargo-test-hadamard-pg18-bench-runtime-attempt.log`.

## Review Focus

Please verify the runtime-detected wrappers preserve the AVX2/FMA and NEON
target-feature contracts while removing unnecessary unsafe from public/test
dispatch surfaces.
