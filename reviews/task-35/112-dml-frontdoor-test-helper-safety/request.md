# Task 35 Packet 112: DML Frontdoor Test Helper Safety

## Code Under Review

- Commit: `04b1e061eb6d095b88de57b2e095aba1198fbe90`
- Scope: `src/tests/dml_frontdoor.rs` plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears `src/tests/dml_frontdoor.rs` by routing repeated analyzed-query and DML frontdoor helper calls through one documented local macro, with direct comments for raw PostgreSQL expression-node inspection and the one bound-parameter list block.

## Result

- Global unsafe-comment baseline moved from `281` entries across `31` files to `252` entries across `30` files.
- `src/tests/dml_frontdoor.rs` moved from `29` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `281` entries, with `29` in `src/tests/dml_frontdoor.rs`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `252` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `252` entries across `30` files.
- `artifacts/dml-frontdoor-baseline-after.log`: `src/tests/dml_frontdoor.rs` residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
