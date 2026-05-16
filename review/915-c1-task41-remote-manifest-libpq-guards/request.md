# Review Request: Task 41 remote manifest libpq guards

Code commit: `a9845c751d54ec7334e7cef7549154b1825973d4`

## Summary

This packet continues Task 41 by migrating the next SPIRE remote manifest
libpq/payload cluster in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated remote epoch manifest publication summary validation to the guard.
- Migrated libpq request plan and summary validation to the guard.
- Migrated manifest payload plan and summary validation to the guard.
- Updated `scripts/unsafe_comment_baseline.txt`.

These functions only needed the relation to validate that `index_oid` is a
SPIRE index before doing SPI-only catalog reads. The guard is scoped to preserve
the old explicit close point before those SPI queries.

## Baseline

- Before: 4652 entries.
- After: 4642 entries.
- Net change: 10 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm each migrated function still validates `index_oid` before any SPI
  query uses it.
- Confirm the guard scope releases `AccessShareLock` before the SPI catalog
  reads, matching the prior explicit `index_close` placement.

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
