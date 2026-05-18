# Task 41 Invariant 2 Review Request: HNSW graph page tuple byte views

## Scope

This packet covers the HNSW graph read helper slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `258d9d28d431e9f84490fac0f50dee151941d3b8` - `Scope HNSW graph page tuple byte views`

Changed file:

- `src/am/ec_hnsw/graph.rs`

## Change Summary

- Added private `with_page_tuple_bytes` for the graph read helpers.
- Routed both relation-opened and caller-buffer graph tuple reads through the same byte-view helper.
- Preserved existing public callback APIs such as `with_graph_storage_tuple` and `with_graph_storage_tuple_from_buffer`.

## Lifetime Argument

The graph APIs already expose tuple bytes only through decode callbacks. This change removes duplicated raw slice construction from `read_page_tuple` and `read_page_tuple_from_buffer`, leaving one helper that creates the slice and immediately invokes the decoder while the page buffer is still available.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the helper preserves the prior error paths and keeps tuple-byte borrows scoped to graph decode callbacks.
