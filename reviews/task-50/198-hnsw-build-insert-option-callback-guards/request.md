# Task 50 Review Request: HNSW Build, Insert, and Options Callback Guards

## Summary

This packet converts the remaining direct `pgrx_extern_c_guard` callback wrappers in HNSW build, insert, and options code to the shared `pg_am_callback!` guard.

The callback bodies are behavior-preserving. The change removes hand-written C-boundary unwind guards from:

- `ec_hnsw_build_callback`
- `ec_hnsw_ambuild`
- `ec_hnsw_ambuildempty`
- `ec_hnsw_aminsert`
- `ec_hnsw_amoptions`

Raw PostgreSQL pointer work stays inside AM callback guard scope; the explicit unsafe boundary is now the shared callback macro contract rather than each local wrapper.

## Code Under Review

- Code commit: `3acd45a7 Centralize HNSW build insert option callbacks`
- Files changed:
  - `src/am/ec_hnsw/build.rs`
  - `src/am/ec_hnsw/insert.rs`
  - `src/am/ec_hnsw/options.rs`

## Unsafe Ledger

- Touched files combined: `unsafe` matches `163 -> 158`
- `src/`: `unsafe` matches `2640 -> 2635`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_hnsw/options.rs src/am/ec_hnsw/insert.rs src/am/ec_hnsw/build.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_hnsw --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass

## Review Focus

- Confirm each converted function is an actual PostgreSQL AM callback entry point appropriate for `pg_am_callback!`.
- Confirm `ec_hnsw_build_callback` still keeps the callback-state cast and tuple array decoding inside the guard.
- Confirm `ec_hnsw_ambuild` and `ec_hnsw_aminsert` control flow is unchanged apart from removing the hand-written guard wrapper.
