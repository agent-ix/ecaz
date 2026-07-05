# Task 41 invariant #2 review request: HNSW vacuum read tuple bytes

## Scope

This packet covers commit `d0b6c08a39f83ea39c44b6371270a74381b73c5b`
(`Scope HNSW vacuum read page tuple bytes`).

The slice completes the HNSW vacuum page tuple byte-view cleanup in
`src/am/ec_hnsw/vacuum.rs` by routing the remaining read-only tuple scans
through scoped tuple-byte helpers:

- pass1 element vacuum planning
- repair request collection
- linear repair candidate collection
- same-page grouped rerank payload loading
- pass2 neighbor repair planning
- fully-dead element finalization planning

The existing helper/copy rewrite surface from packet 144 remains the only raw
tuple-byte access in `vacuum.rs`.

## Reviewer focus

- Confirm all read-only tuple bytes are consumed inside
  `shared::with_page_line_tuple_bytes` closures.
- Confirm nonmatching tuple tags still skip the line pointer instead of
  decoding the wrong tuple kind.
- Confirm same-page grouped rerank payload loading still falls back to the
  existing cross-page graph loader when the rerank tuple is on another block.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check HEAD -- src/am/ec_hnsw/vacuum.rs` passed.
