# Task 35 Packet 084: HNSW Options Safety Comments

## Code Under Review

- Commit: `3004fc4a9dfe51a1670a1a6aa51d94b7874fa3b7`
- Files:
  - `src/am/ec_hnsw/options.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Scope

This slice restarts the HNSW residual burndown after SPIRE closeout by clearing the HNSW reloptions surface.

The added comments document safety boundaries for:

- PostgreSQL `amoptions` callback entry through `pgrx_extern_c_guard`;
- `build_local_reloptions` ownership of the returned `bytea`;
- string reloption offsets into PostgreSQL's `rd_options` varlena blob;
- NUL-terminated string reloption storage;
- relation `rd_options` reads from an open HNSW index relation;
- casting `rd_options` to the `TqHnswReloptions` layout produced by `ec_hnsw_amoptions`.

## Baseline Movement

- Global unsafe-comment baseline: `1768 -> 1760`
- `src/am/ec_hnsw/options.rs`: `8 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `1760` entries across `50` files.
- `artifacts/hnsw-options-baseline-after.log`: HNSW options entries are `0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known unrelated warnings remain:

- unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
- unused SPIRE imports/re-exports in `src/am/mod.rs`.

`cargo fmt --all` emitted the repo's existing stable-rustfmt warnings for unstable rustfmt options. It also touched `hardening/careful/src/lib.rs` and `src/quant/simd.rs`; those unrelated formatting changes were restored before commit.
