# Task 111f Review Request: Dead IVF Dense Format Cleanup

Code commit: `69bdeecf14ee2f8cd256b90168039b8acf5e0b49`

## Summary

This strips the abandoned Task 111 investigation formats before the 111 lane
merges to `main`:

- Removed the page-spanning packed dense format: tags, tuple/ref types, build
  writer, scan assembly, vacuum, reloption, and tests.
- Removed the columnar frozen-list format and page-scatter scorer: tag, tuple
  types, page-run/pinned-page reader, scan/vacuum/build paths, reloption/GUC,
  EXPLAIN counters, and tests.
- Kept the survivor paths: row postings, dense posting blocks, aligned dense
  posting blocks, dense coalescing/typed views, and `coarse_rerank`.
- Updated `docs/on-disk-format.md` to list only the surviving IVF tuple tags.
- Added `spec/adr/ADR-078-ivf-dense-format-negative-result.md` to preserve the
  negative-result rationale.

## Review Focus

Please check that no keeper behavior was accidentally changed while removing the
dead branches, especially:

- Dense block and aligned dense block write/read byte layout.
- Scan dispatch for row and dense entries after the enum/tag reduction.
- Vacuum rewrite behavior for row and dense blocks.
- Reloption/admin snapshot behavior for `coarse_rerank`, `dense_posting_blocks`,
  and `dense_posting_typed_layout`.

## Evidence

See `artifacts/manifest.md` for commands and key result lines.

- `artifacts/cargo-check-pg18.log`: `cargo check --no-default-features --features pg18` passed.
- `artifacts/cargo-clippy-pg18.log`: `cargo clippy --no-default-features --features pg18 -- -D warnings` passed.
- `artifacts/cargo-test-ivf-explain-pg18.log`: IVF EXPLAIN counter/property unit tests passed, 2/2.
- `artifacts/cargo-pgrx-test-pg18-ivf-dense.log`: dense PG18 pgrx fixtures passed, 6/6.
- `artifacts/cargo-pgrx-test-pg18-coarse-rerank.log`: coarse-rerank PG18 pgrx fixture passed, 1/1.

Forbidden source/doc symbol sweep over `src` and `docs/on-disk-format.md` had
no matches for the removed tags, types, GUCs, reloptions, or EXPLAIN counters.
