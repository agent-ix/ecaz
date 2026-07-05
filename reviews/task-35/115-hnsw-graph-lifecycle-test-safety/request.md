# Task 35 Packet 115: HNSW Graph Lifecycle Test Safety

## Code Under Review

- Commit: `03a741477995f83e728724a4833e09fa88020220`
- Scope: `src/tests/ec_hnsw_graph_lifecycle.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/ec_hnsw_graph_lifecycle.rs` by routing HNSW metadata, page, vacuum, frontier, and scan debug helper calls through one documented local macro.

## Result

- Global unsafe-comment baseline moved from `196` entries across `28` files to `169` entries across `27` files.
- `src/tests/ec_hnsw_graph_lifecycle.rs` moved from `27` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `196` entries, with `27` in `src/tests/ec_hnsw_graph_lifecycle.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `169` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `169` entries across `27` files.
- `artifacts/hnsw-graph-lifecycle-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
