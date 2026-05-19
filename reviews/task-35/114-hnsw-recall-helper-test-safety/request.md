# Task 35 Packet 114: HNSW Recall Helper Test Safety

## Code Under Review

- Commit: `c67fe59c9b697939ad92f28a5b2285330edf8521`
- Scope: `src/tests/ec_hnsw_recall_helpers.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/ec_hnsw_recall_helpers.rs` by routing repeated HNSW recall/debug helper calls through one documented local macro and documenting the remaining guarded relation block-count call inline.

## Result

- Global unsafe-comment baseline moved from `224` entries across `29` files to `196` entries across `28` files.
- `src/tests/ec_hnsw_recall_helpers.rs` moved from `28` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `224` entries, with `28` in `src/tests/ec_hnsw_recall_helpers.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `196` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `196` entries across `28` files.
- `artifacts/hnsw-recall-helpers-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
