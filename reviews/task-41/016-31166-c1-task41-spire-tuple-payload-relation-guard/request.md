# Review Request: Task 41 SPIRE tuple-payload relation guard

Code commit: `56091e5e7d7017fda12072738c952af0466cc03b`

## Summary

This packet continues Task 41 by consolidating repeated raw SPIRE index relation
opens used only to read the heap relation OID for remote tuple-payload
entrypoints.

- Added `ec_spire_index_heap_relation_oid`, which opens a validated SPIRE index
  guard, copies `rd_index.indrelid`, and then drops the guard before SPI work.
- Migrated insert, update, delete, and select remote tuple-payload entrypoints
  to the helper.
- Removed duplicated raw `open_valid_ec_spire_index` / `index_close` pairs and
  repeated `rd_index` dereferences from those four entrypoints.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,479 entries.
- After: 4,467 entries.
- Net change: 12 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm `ec_spire_index_heap_relation_oid` copies only the heap relation OID
  while the guard is live and does not let the raw relation pointer escape.
- Confirm all tuple-payload SPI work still happens after the guard has dropped.
- Confirm the helper is appropriate for validation-only/metadata-copy callsites
  under the Task 41 strategy note.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check 56091e5e^ 56091e5e`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
