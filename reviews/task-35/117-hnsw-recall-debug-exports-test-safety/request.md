# Task 35 Packet 117: HNSW Recall Debug Export Test Safety

## Code Under Review

- Commit: `cb8551fab7228d3861dd279766ada3f74bcb4ba9`
- Scope: `src/tests/ec_hnsw_recall_debug_exports.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/ec_hnsw_recall_debug_exports.rs` by routing SQL-visible HNSW test/debug export calls through one documented local macro after the exported helpers validate their HNSW index OID.

## Result

- Global unsafe-comment baseline moved from `150` entries across `26` files to `132` entries across `25` files.
- `src/tests/ec_hnsw_recall_debug_exports.rs` moved from `18` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `150` entries, with `18` in `src/tests/ec_hnsw_recall_debug_exports.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `132` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `132` entries across `25` files.
- `artifacts/hnsw-recall-debug-exports-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
