# Task 35 Packet 113: HNSW Runtime Profile Test Helper Safety

## Code Under Review

- Commit: `d5016bad396538a455d955ba126ef8d1fcb8a761`
- Scope: `src/tests/ec_hnsw_runtime_profiles.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/ec_hnsw_runtime_profiles.rs` by routing repeated HNSW runtime/profile debug helper calls through one documented local macro.

## Result

- Global unsafe-comment baseline moved from `252` entries across `30` files to `224` entries across `29` files.
- `src/tests/ec_hnsw_runtime_profiles.rs` moved from `28` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `252` entries, with `28` in `src/tests/ec_hnsw_runtime_profiles.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `224` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `224` entries across `29` files.
- `artifacts/hnsw-runtime-profiles-baseline-after.log`: file residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
