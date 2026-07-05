# Task 50 Review Request: HNSW Source Datum/SIMD Unsafe Reduction

## Summary

This packet handles `src/am/ec_hnsw/source.rs`, the HNSW source top-15 file.

Code commit:

- `a18f492c Reduce HNSW source unsafe blocks`

The change consolidates repeated unsafe regions around SIMD lane loads, PostgreSQL formatted type-name ownership, and tuple-slot datum materialization.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/source.rs` | 78 | 52 | <=54 | met |

## Review Notes

- AVX2/FMA and NEON scoring behavior is unchanged; repeated lane loads now share one documented block per loop shape.
- Type-name resolution still unwraps base types, copies the PostgreSQL-owned formatted string, and frees it before returning.
- Slot datum access still materializes the descriptor-backed attribute, rejects NULL values, and returns the checked datum.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/source.rs'` passed with count 52.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/source.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes unsafe ownership structure but not HNSW scoring arithmetic, tuple selection, or storage layout.
