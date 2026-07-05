# Task 35 Packet 119: Remote Search Test Safety

## Code Under Review

- Commit: `72b366b144aa2f584a119585ef66c03b5fbe292a`
- Scope: `src/tests/remote_search/*` residual baseline plus `scripts/unsafe_comment_baseline.txt`
- Packet type: unsafe-comment burndown code slice

## Scope

This packet clears the remote-search test baseline by documenting SPiRE test-only debug hooks that rewrite placement, consistency, and catalog state after each fixture derives the target pids/epochs from local snapshots. It also documents the two scoped PostgreSQL interrupt/timeout signal guards.

## Result

- Global unsafe-comment baseline moved from `119` entries across `23` files to `60` entries across `13` files.
- `src/tests/remote_search/*` moved from `59` entries to `0`.
- Remaining baseline entries are still test-only under `src/tests/`.

## Validation

- `artifacts/unsafe-baseline-report-before.log`: pre-slice baseline was `119` entries, with `59` under `src/tests/remote_search/`.
- `artifacts/unsafe-baseline-update-after-format.log`: regenerated baseline after formatting, resulting in `60` entries.
- `artifacts/unsafe-baseline-report-after.log`: post-slice baseline is `60` entries across `13` files.
- `artifacts/remote-search-baseline-after.log`: remote-search residual is `0`.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
