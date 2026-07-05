# Task 35 Packet 109: HNSW Scan Gettuple Test Debug Helper Safety

## Code Under Review

- Commit: `b86d7b2bde79867915d51ed2202fb8fafd09b600`
- Scope: `src/tests/ec_hnsw_scan_gettuple.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears the `src/tests/ec_hnsw_scan_gettuple.rs` unsafe-comment baseline by routing repeated `am::debug_*` scan helper calls through one documented test-only macro.

The macro captures the shared invariant for these tests: each `pg_test` creates the referenced HNSW index before calling the extension scan debug helper, and the helper owns PostgreSQL relation and scan access for the supplied OID.

## Result

- Global unsafe-comment baseline moved from `416` entries across `34` files to `362` entries across `33` files.
- `src/tests/ec_hnsw_scan_gettuple.rs` moved from `54` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `416` entries, with `54` in `src/tests/ec_hnsw_scan_gettuple.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `362` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `362` entries across `33` files.
- `artifacts/hnsw-scan-gettuple-baseline-after.log`: `src/tests/ec_hnsw_scan_gettuple.rs` residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
