# Task 35 Packet 116: HNSW Build Test Safety

## Code Under Review

- Commit: `704eeb8e38db9514afcc1c46fc3d5a1c348d2760`
- Scope: `src/tests/ec_hnsw_build.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/ec_hnsw_build.rs` by routing HNSW index page debug helpers and relation-backed graph/codebook reads through one documented local macro.

## Result

- Global unsafe-comment baseline moved from `169` entries across `27` files to `150` entries across `26` files.
- `src/tests/ec_hnsw_build.rs` moved from `19` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `169` entries, with `19` in `src/tests/ec_hnsw_build.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `150` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `150` entries across `26` files.
- `artifacts/hnsw-build-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
