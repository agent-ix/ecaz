# Task 41 invariant #2 review request: HNSW scan debug tuple bytes

## Scope

This packet covers commit `b24cc194801d71069a8f87363a4dfcac8f6d936a`
(`Scope HNSW scan debug page tuple bytes`).

The slice keeps the debug-only HNSW scan page tuple byte view local to
`src/am/ec_hnsw/scan_debug.rs` by adding
`debug_with_page_line_tuple_bytes` and routing the three debug collection
loops through it.

The helper preserves the previous debug behavior: unused, zero-length, or
out-of-bounds line pointers are skipped rather than treated as hard errors.

## Reviewer focus

- Confirm tuple byte views are only used inside the debug helper closure.
- Confirm the three collection loops still filter by the storage-specific
  element tag before calling `graph::load_exact_graph_element`.
- Confirm invalid debug line pointers remain skipped.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check HEAD -- src/am/ec_hnsw/scan_debug.rs` passed.
