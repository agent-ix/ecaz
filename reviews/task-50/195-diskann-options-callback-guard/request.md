# Review Request: DiskANN Options Callback Guard

## Summary

Replaced the hand-written `pgrx_extern_c_guard` wrapper in `ec_diskann_amoptions` with the shared `pg_am_callback!` boundary helper.

The reloption registration logic is unchanged. This only centralizes the PostgreSQL callback unwind guard and removes one direct unsafe block from the DiskANN options callback.

## Unsafe Ledger

- `src/am/ec_diskann/options.rs`: `8 -> 7`
- `src/`: `2647 -> 2646`

## Validation

- `rustfmt --check src/am/ec_diskann/options.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`
- `cargo test --lib am::ec_diskann::options --no-default-features --features pg18,pg_test --no-run`

Artifact logs and command metadata are in `artifacts/manifest.md`.
