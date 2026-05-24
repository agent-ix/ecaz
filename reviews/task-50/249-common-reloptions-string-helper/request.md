# Task 50 Review Request: Common Reloptions String Helper

## Summary

Centralized the repeated AM reloptions string decode path behind
`src/am/common/reloptions.rs`.

The helper remains `unsafe fn` because callers must still prove that the
relation-owned reloptions blob and string offset belong to the matching
PostgreSQL reloptions layout. Each typed AM reloptions view now has one
explicit call-site boundary instead of duplicating pointer arithmetic and
`CStr::from_ptr` handling.

Affected AMs:

- DiskANN
- HNSW
- IVF
- SPIRE

## Unsafe Burndown

- repository `src` unsafe grep count: `2441 -> 2440`
- each AM option module drops `7 -> 6`
- new common helper carries the one shared backend contract: `0 -> 3`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/common/reloptions.rs src/am/common/mod.rs src/am/ec_diskann/options.rs src/am/ec_hnsw/options.rs src/am/ec_ivf/options.rs src/am/ec_spire/options/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the helper keeps the reloptions blob/offset contract explicit at
the typed view boundary and preserves the prior AM-specific error text shape.
