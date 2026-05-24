# Review Request: DiskANN Build Callback Guards

## Summary

Replaced three hand-written `pgrx_extern_c_guard` wrappers in DiskANN build callbacks with the shared `pg_am_callback!` boundary helper:

- `ec_diskann_ambuild`
- `ec_diskann_ambuildempty`
- `ec_diskann_build_callback`

This advances Task 50 P1 on DiskANN build wiring. The PostgreSQL relation, `IndexInfo`, Datum/null array, and opaque build-state pointer contracts stay at the callback boundary; only the repeated unwind guard is centralized.

## Unsafe Ledger

- `src/am/ec_diskann/ambuild.rs`: `41 -> 38`
- `src/`: `2650 -> 2647`

## Validation

- `rustfmt --check src/am/ec_diskann/ambuild.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::ec_diskann::ambuild --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
