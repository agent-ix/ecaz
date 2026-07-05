# Review Request: Task 41 remote manifest executor guards

Code commit: `a090e5f38acd9e772c12651dce2301f77ab83e8b`

## Summary

This packet continues Task 41 by migrating the next SPIRE remote manifest
executor cluster in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated remote manifest payload validation and apply entrypoints.
- Migrated manifest libpq dispatch and bind plan/summary entrypoints.
- Migrated manifest libpq executor work plan/summary entrypoints.
- Updated `scripts/unsafe_comment_baseline.txt`.

The migrated entrypoints use the relation only to validate that the supplied
OID is a SPIRE index before parsing payloads or reading/writing SPI catalog
state. Each guard is scoped to drop before that follow-on work, matching the
old explicit `index_close` point.

## Baseline

- Before: 4642 entries.
- After: 4628 entries.
- Net change: 14 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the validation/apply entrypoints still reject invalid index OIDs
  before payload parsing or SPI writes.
- Confirm the dispatch/bind/executor functions release the relation guard
  before SPI-only diagnostic reads.

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
