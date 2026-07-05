# Review Request: SPIRE Build Callback Guards

## Summary

Centralized SPIRE build callback unwind guard boundaries by replacing three hand-written `unsafe { pgrx::pgrx_extern_c_guard(...) }` wrappers in `src/am/ec_spire/build/tuples.rs` with the shared `pg_am_callback!` macro.

Touched callback surfaces:

- `ec_spire_ambuild`
- `ec_spire_ambuildempty`
- `ec_spire_build_callback`

This advances Task 50 P1 on a SPIRE production build path. The callback bodies and their PostgreSQL pointer contracts are unchanged; the unsafe unwind boundary now uses the same shared AM callback abstraction already used elsewhere.

## Unsafe Ledger

- `src/am/ec_spire/build/tuples.rs`: `11 -> 8`
- `src/am/ec_spire/build.rs`: `0 -> 0`
- `src/`: `2660 -> 2657`

The diff is mostly reindentation after removing one guard-wrapper nesting level.

## Validation

- `rustfmt --check src/am/ec_spire/build.rs src/am/ec_spire/build/tuples.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::ec_spire::build --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
