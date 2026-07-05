# Task 35 Packet 111: Shared Test Helper Safety

## Code Under Review

- Commit: `8c953f8978405feb67e63f5e6e6dd0dba11ed510`
- Scope: `src/tests/mod.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet documents shared test helper unsafe boundaries in `src/tests/mod.rs`, including:

- PostgreSQL backend symbol lookup for query-cancel and statement-timeout fixtures;
- timeout function-pointer transmute and guard restoration;
- SPIRE placement rewrite test SQL wrappers;
- SPIRE coordinator insert prepare helpers guarded by an open relation;
- shared HNSW/IVF debug scan/page helpers;
- recall block-count probes;
- PostgreSQL parse/analyze helpers used by custom scan planner tests.

## Result

- Global unsafe-comment baseline moved from `321` entries across `32` files to `281` entries across `31` files.
- `src/tests/mod.rs` moved from `40` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `321` entries, with `40` in `src/tests/mod.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `281` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `281` entries across `31` files.
- `artifacts/tests-mod-baseline-after.log`: `src/tests/mod.rs` residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
