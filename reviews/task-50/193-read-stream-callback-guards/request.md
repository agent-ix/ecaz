# Review Request: Read Stream Callback Guards

## Summary

Replaced three hand-written `pgrx_extern_c_guard` wrappers in PG18 read-stream callbacks with the shared `pg_callback!` boundary helper:

- `graph_prefetch_cb`
- `linear_prefetch_cb`
- `block_sequence_prefetch_cb`

The callback-private pointer casts remain at the read-stream callback boundary with their existing state-type invariants. This slice only centralizes the unwind guard and removes repeated direct unsafe guard blocks from `src/am/common/stream.rs`.

## Unsafe Ledger

- `src/am/common/stream.rs`: `26 -> 23`
- `src/`: `2653 -> 2650`

## Validation

- `rustfmt --check src/am/common/stream.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::common::stream --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
