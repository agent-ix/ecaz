# Review Request: Task 41 SPIRE production diagnostic relation guards

Code commit: `d26361e73a785b3aeefc0fa98cc256964fc0ccbb`

## Summary

This packet continues Task 41 by migrating the next SPIRE production diagnostic
cluster in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated production policy, executor, degraded-skip, scan-handoff,
  heap-resolution, read-profile, operator, endpoint-identity, and libpq
  readiness diagnostics.
- Replaced direct `open_valid_ec_spire_index` / `index_close` pairs with guard
  ownership and `index_relation.as_ptr()` at AM helper boundaries.
- Preserved the old lock lifetime by dropping the guard after AM helpers return
  copied summary rows or owned row vectors and before result shaping.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,517 entries.
- After: 4,493 entries.
- Net change: 24 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm every AM helper receives `index_relation.as_ptr()` only while the
  guard is live.
- Confirm each explicit `drop(index_relation)` happens after the AM helper has
  copied the data it returns and before iterator/result shaping.
- Confirm this slice did not move SPI, environment-variable, or libpq connection
  work under a live relation guard.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check d26361e7^ d26361e7`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
