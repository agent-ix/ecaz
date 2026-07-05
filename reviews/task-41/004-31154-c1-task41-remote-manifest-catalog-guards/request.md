# Review Request: Task 41 remote manifest catalog guards

Code commit: `05c9f2f293317f1b343796d5c280945f3986320d`

## Summary

This packet continues Task 41 by migrating the next SPIRE remote manifest
catalog cluster in `src/lib.rs` to `AccessShareIndexRelation`.

- Migrated manifest entry catalog validation to the guard.
- Migrated manifest catalog summary AM reads to the guard, dropping it before
  the subsequent SPI catalog reads.
- Migrated manifest freshness validation to the guard.
- Migrated manifest publication plan AM reads to the guard, dropping it before
  the subsequent SPI catalog reads.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4660 entries.
- After: 4652 entries.
- Net change: 8 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the catalog summary/publication functions keep the relation live only
  for AM reads and release it before SPI-only catalog scans, matching the old
  explicit close point.
- Confirm validation-only catalog functions intentionally open and drop the
  guard before building SQL from the validated OID.

## Validation

- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-914.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
