# Review Request: SPIRE Phase 13e Remote Placement Smoke Gate

## Summary

This checkpoint adds the first production CLI smoke gate for the Phase 13e
distributed-read path. `ecaz bench spire-pipeline` can now fail explicitly when
the resolved SPIRE index has no remote placements, so AWS/local distributed
verification cannot accidentally proceed against an empty or local-only
placement directory.

Code commit: `7b5ea98b9fde3e65bfc4171fbd54367bc86dfbbb`

## Changes

- Added `--require-remote-placements` to `ecaz bench spire-pipeline`.
- The gate queries `ec_spire_index_placement_snapshot($index)` and treats
  `node_id > 1` as remote placement ownership.
- Empty placement directories and local-only placement directories fail before
  the report runs.
- The error reports total placement count, local placement count, and remote
  node count to make AWS smoke failures actionable.

## Validation

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli spire_pipeline`
- Result: 16 passed, 0 failed

## Scope Notes

This does not materialize remote PostgreSQL shards or prove distributed query
recall. It closes the specific Phase 13e acceptance gap that empty or local-only
placement directories must fail the distributed smoke gate.
