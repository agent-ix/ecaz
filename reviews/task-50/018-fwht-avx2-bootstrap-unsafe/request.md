# Task 50 Review Request: FWHT AVX2 Bootstrap Unsafe Reduction

## Summary

This packet handles the remaining RaBitQ/shared-quant top-15 file, `src/quant/hadamard.rs`.

Code commit:

- `9043ecf0 Consolidate FWHT AVX2 bootstrap unsafe blocks`

The change consolidates adjacent AVX2 bootstrap load/transform/store operations into one documented unsafe region per chunk width. It does not change dispatch, tiling, SIMD instruction selection, store order, or scalar fallback behavior.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/quant/hadamard.rs` | 62 | 43 | <=43 | met |

## Review Notes

- The 8-, 16-, 32-, and 64-lane AVX2 bootstrap cases already had a single chunk-size proof per case.
- This slice keeps those proofs local but removes the repeated per-operation unsafe acknowledgements inside the same proof region.
- The transformed data path is unchanged: each case loads the same lanes, calls the same FWHT block helper, and stores to the same offsets.

## Validation

- `make unsafe-block-count PATHS='src/quant/hadamard.rs'` passed with count 43.
- `rustfmt --edition 2021 --check src/quant/hadamard.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `cargo test hadamard --lib --no-default-features --features pg18` compiled, then failed before running the filtered tests with the existing local pgrx runtime linker issue: `undefined symbol: CacheRegisterRelcacheCallback`.

No benchmark result is claimed in this packet. The slice only consolidates unsafe regions around identical AVX2 bootstrap operations.
