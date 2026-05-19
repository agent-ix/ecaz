# Task 35 Packet 118: HNSW Runtime Comparison Test Safety

## Code Under Review

- Commit: `b9c3be1c1d1eef590b556cdf5d436ba77671c503`
- Scope: `src/tests/ec_hnsw_runtime_comparisons.rs`, `src/tests/hnsw_misc.rs`, and `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears the remaining HNSW-specific test baseline by routing runtime comparison debug calls through one documented local macro and documenting the binary receive rejection fixtures in `hnsw_misc.rs`.

## Result

- Global unsafe-comment baseline moved from `132` entries across `25` files to `119` entries across `23` files.
- `src/tests/ec_hnsw_runtime_comparisons.rs` moved from `11` entries to `0`.
- `src/tests/hnsw_misc.rs` moved from `2` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `132` entries, with `11` in `src/tests/ec_hnsw_runtime_comparisons.rs` and `2` in `src/tests/hnsw_misc.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `119` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `119` entries across `23` files.
- `artifacts/hnsw-runtime-comparison-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
