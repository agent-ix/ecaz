# Task 41 Invariant 2 Review Request: HNSW scan page tuple byte views

## Scope

This packet covers the HNSW linear scan page tuple slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `047b069041d0f1771ae3c9e23801a491194ecdf6` - `Scope HNSW scan page tuple byte views`

Changed files:

- `src/am/ec_hnsw/shared.rs`
- `src/am/ec_hnsw/scan.rs`

## Change Summary

- Exposed the shared page-line tuple helper to sibling HNSW modules.
- Routed the linear scan tuple decode path through `shared::with_page_line_tuple_bytes`.
- Preserved skipped-element counter behavior for unused, non-element, deleted, and empty-heaptid tuples.

## Lifetime Argument

The scan page tuple bytes are now created by the shared helper and consumed inside a callback while the scan buffer remains share-locked. The callback returns an owned decoded tuple value for scoring; no borrowed page slice escapes the helper.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether scan skip/scoring behavior is preserved and whether the helper use is appropriate for a manually locked `pg_sys::Buffer` scan path.
