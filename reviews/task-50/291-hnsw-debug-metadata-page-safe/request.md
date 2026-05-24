# Review Request: HNSW Debug Metadata Page Safe Boundary

## Summary

This slice continues the HNSW debug unsafe burndown after packet 290.

Code commit: `36e7b9ff2a6792c30512bda4bda51fd88f20db1f`

Changes:

- Added safe private `debug_read_metadata_page`.
- Centralized the raw `shared::read_metadata_page` call inside that helper.
- Converted all HNSW debug metadata readers in `scan_debug.rs` to use the helper.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2116 -> 2106`.
- Removed eleven caller-local metadata page unsafe blocks and replaced them with one named debug boundary.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/291-hnsw-debug-metadata-page-safe/artifacts/manifest.md`

