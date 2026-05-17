# Review Request: Task 41 SPIRE executor relation guards

Code commit: `cd729bb6abb794874e4e8d9e8b3e779ed5afabef`

## Summary

This packet continues Task 41 by migrating the next adjacent SPIRE libpq
executor diagnostic cluster in `src/lib.rs` from raw `Relation` open/close
pairs to `AccessShareIndexRelation`.

- Migrated libpq connection-open diagnostics.
- Migrated executor connection-check, candidate, receive-attempt, heap-candidate,
  identity-cache, work-plan, work-summary, and dispatch-summary helpers.
- Preserved the prior lock lifetime by dropping the guard after AM helpers
  return owned rows or summary rows.
- Kept environment-variable connection checks and summary aggregation outside
  the relation guard scope.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,539 entries.
- After: 4,517 entries.
- Net change: 22 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm AM helpers only receive `index_relation.as_ptr()` while the guard is
  live.
- Confirm `ec_spire_remote_search_libpq_executor_work_plan` and
  `ec_spire_remote_search_libpq_executor_work_summary` keep the guard live
  across both AM calls and drop it before iterator shaping.
- Confirm connection-check environment lookup and libpq connection probing still
  happen after the index relation has been closed.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `make unsafe-baseline-report`
  - artifact: `artifacts/baseline-after.log`
