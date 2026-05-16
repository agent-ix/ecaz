# Review Request: Task 41 remote manifest result guards

Code commit: `cab0c2f4e465e2ca4930d694e4fd2066f4a49273`

## Summary

This packet continues Task 41 by migrating the next SPIRE remote manifest
result/publication cluster in `src/lib.rs` from raw `Relation` open/close pairs
to `AccessShareIndexRelation`.

- Migrated manifest libpq dispatch summary and executor readiness entrypoints.
- Migrated manifest libpq receive plan and summary entrypoints.
- Migrated manifest libpq executor results validation.
- Migrated manifest publication gate/result summary entrypoints.
- Updated `scripts/unsafe_comment_baseline.txt`.

The migrated functions validate that `index_oid` is a SPIRE index, then drop the
guard before SPI-only reads or libpq executor work.

## Baseline

- Before: 4628 entries.
- After: 4614 entries.
- Net change: 14 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm all migrated functions preserve the old validate-before-use behavior.
- Confirm the scoped guard releases the relation before SPI catalog reads and
  before external libpq executor work in the results function.

## Validation

- `bash scripts/unsafe_baseline_report.sh artifacts/unsafe-baseline-before.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
