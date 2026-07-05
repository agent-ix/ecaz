# Task 35 Packet 110: HNSW Storage Lifecycle Test Debug Helper Safety

## Code Under Review

- Commit: `b39752e5537e35af3a5b3a4283594d370a360485`
- Scope: `src/tests/ec_hnsw_storage_lifecycle.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears the `src/tests/ec_hnsw_storage_lifecycle.rs` unsafe-comment baseline by routing repeated `am::debug_*` storage helper calls through one documented test-only macro, plus two direct comments for grouped graph/rerank payload loads tied to an open relation guard.

The macro captures the shared invariant for these tests: each `pg_test` creates the referenced HNSW index before calling the extension storage debug helper, and the helper owns PostgreSQL relation/page access for the supplied OID.

## Result

- Global unsafe-comment baseline moved from `362` entries across `33` files to `321` entries across `32` files.
- `src/tests/ec_hnsw_storage_lifecycle.rs` moved from `41` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `362` entries, with `41` in `src/tests/ec_hnsw_storage_lifecycle.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `321` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `321` entries across `32` files.
- `artifacts/hnsw-storage-lifecycle-baseline-after.log`: `src/tests/ec_hnsw_storage_lifecycle.rs` residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
