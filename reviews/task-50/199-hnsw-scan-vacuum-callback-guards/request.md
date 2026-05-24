# Task 50 Review Request: HNSW Scan and Vacuum Callback Guards

## Summary

This packet converts HNSW scan and vacuum AM callback wrappers from hand-written `pgrx_extern_c_guard` blocks to the shared `pg_am_callback!` guard.

Converted callbacks:

- `ec_hnsw_ambeginscan`
- `ec_hnsw_amrescan`
- `ec_hnsw_amgettuple`
- `ec_hnsw_amendscan`
- `ec_hnsw_ambulkdelete`
- `ec_hnsw_amvacuumcleanup`

The remaining explicit `pgrx_extern_c_guard` in `src/am/ec_hnsw/vacuum.rs` is the pg_test-only debug dead-TID callback and is intentionally left for a separate non-AM callback pass.

## Code Under Review

- Code commit: `495b53f0 Centralize HNSW scan vacuum callbacks`
- Files changed:
  - `src/am/ec_hnsw/scan.rs`
  - `src/am/ec_hnsw/vacuum.rs`

## Unsafe Ledger

- Touched files combined: `unsafe` matches `309 -> 303`
- `src/`: `unsafe` matches `2635 -> 2629`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_hnsw/scan.rs src/am/ec_hnsw/vacuum.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_hnsw --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass

## Review Focus

- Confirm all converted functions are HNSW AM callback entry points appropriate for `pg_am_callback!`.
- Confirm scan opaque allocation, rescan state reset, gettuple output, and endscan cleanup remain inside the callback guard.
- Confirm vacuum callbacks still preserve the no-op stats path when PostgreSQL supplies no bulk-delete callback.
