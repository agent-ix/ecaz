# Task 50 Review Request: Remaining Callback Guard Closeout

## Summary

This packet removes the remaining direct `pgrx_extern_c_guard` wrappers from the HNSW/DiskANN callback surface outside the shared callback helper module.

Converted call sites:

- `src/am/ec_hnsw/build_parallel.rs`
  - `ec_hnsw_parallel_build_callback` now uses `pg_am_callback!`
  - `ec_hnsw_parallel_build_main` now uses `pg_callback!`
  - `ec_hnsw_parallel_graph_build_main` now uses `pg_callback!`
- `src/am/ec_hnsw/vacuum.rs`
  - pg_test debug vacuum dead-TID callback now uses `pg_callback!`
- `src/am/ec_diskann/routine.rs`
  - pg_test debug vacuum dead-TID callback now uses `pg_callback!`

The direct guard scan for `src/am/ec_hnsw`, `src/am/ec_diskann/routine.rs`, `src/am/ec_spire`, and `src/am/common` now reports `pgrx_extern_c_guard` only in `src/am/common/callback.rs`.

## Code Under Review

- Code commit: `14dc7023 Centralize remaining callback guards`
- Files changed:
  - `src/am/ec_hnsw/build_parallel.rs`
  - `src/am/ec_hnsw/vacuum.rs`
  - `src/am/ec_diskann/routine.rs`

## Unsafe Ledger

- Touched files combined: `unsafe` matches `327 -> 322`
- `src/`: `unsafe` matches `2629 -> 2624`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_hnsw/build_parallel.rs src/am/ec_hnsw/vacuum.rs src/am/ec_diskann/routine.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_hnsw --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_diskann --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass
- Direct callback guard scan: only `src/am/common/callback.rs` contains `pgrx_extern_c_guard` in the scanned AM surfaces

## Review Focus

- Confirm the HNSW parallel heap callback is correctly treated as an AM callback and still keeps callback-state tuple work inside the guard.
- Confirm HNSW background-worker entry points are appropriate `pg_callback!` users rather than `pg_am_callback!`.
- Confirm pg_test debug vacuum callbacks in HNSW and DiskANN still dereference callback state only inside the callback guard.
