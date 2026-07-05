# Review Request: Generic PostgreSQL Callback Guard

## Summary

Added a generic `pg_callback!` macro in `src/am/common/callback.rs` for PostgreSQL C callbacks that are not specifically AM entry points, then used it to remove hand-written `pgrx_extern_c_guard` blocks from:

- `src/am/common/parallel.rs`
- `src/am/ec_spire/dml_frontdoor/mod.rs`

`pg_am_callback!` stays self-contained to avoid requiring every existing AM callback caller to import the generic macro too.

## Unsafe Ledger

- `src/am/common/callback.rs`: `2 -> 3`
- `src/am/common/parallel.rs`: `52 -> 48`
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `75 -> 74`
- `src/`: `2657 -> 2653`

The new shared macro adds one named boundary in common code while deleting five direct call-site unsafe rows.

## Validation

- `rustfmt --check src/am/common/callback.rs src/am/common/parallel.rs src/am/ec_spire/dml_frontdoor/mod.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::common::parallel --no-default-features --features pg18,pg_test --no-run`
- `cargo test --lib am::ec_spire::dml_frontdoor --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
