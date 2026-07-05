# Review Request: DiskANN AM Callback Guards

## Summary

Centralized the DiskANN AM callback unwind guard boundary by replacing seven hand-written `unsafe { pgrx::pgrx_extern_c_guard(...) }` wrappers in `src/am/ec_diskann/routine.rs` with the repo-standard `pg_am_callback!` macro.

This keeps the callback contract at the shared guard abstraction used by IVF/SPIRE AM callbacks and removes repeated per-callback unsafe guard blocks without changing DiskANN insert, scan, vacuum, or scan-end behavior.

## Unsafe Ledger

- `src/am/ec_diskann/routine.rs`: `78 -> 71`
- `src/`: `2671 -> 2664`

The diff is mostly reindentation from removing one wrapper nesting level around callback bodies.

## Validation

- `rustfmt --check src/am/ec_diskann/routine.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::ec_diskann::routine --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
