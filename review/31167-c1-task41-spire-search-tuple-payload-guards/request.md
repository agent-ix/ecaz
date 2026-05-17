# Review Request: Task 41 SPIRE search tuple-payload guards

Code commit: `8886896f874e330305776b8edd39594bbde4e436`

## Summary

This packet continues Task 41 by migrating the SPIRE remote-search tuple payload
and typed tuple payload diagnostics from raw relation ownership to guard-backed
metadata and AM access.

- Split heap-relation metadata copying into
  `ec_spire_heap_relation_oid_from_index`, which accepts a live
  `AccessShareIndexRelation` and copies `rd_index.indrelid`.
- Kept one guard per diagnostic because both functions need heap metadata and a
  local heap-candidate AM read from the same validated index.
- Replaced raw `open_valid_ec_spire_index` / `index_close` pairs with
  `open_valid_ec_spire_index_guard`, `index_relation.as_ptr()`, and explicit
  `drop(index_relation)` before SPI tuple-payload fetches.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,467 entries.
- After: 4,461 entries.
- Net change: 6 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm `ec_spire_heap_relation_oid_from_index` only copies `indrelid` while
  the guard is live and does not let the raw relation pointer escape.
- Confirm both tuple-payload diagnostics drop the guard before validating
  payload columns, building heap regclass text, or running SPI fetches.
- Confirm using one guard per function is preferable here because the same
  validated index is needed for metadata and AM candidate rows.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check 8886896f^ 8886896f`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
