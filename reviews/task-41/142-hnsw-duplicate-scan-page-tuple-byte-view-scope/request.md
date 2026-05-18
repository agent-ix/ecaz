# Task 41 Invariant 2 Review Request: HNSW duplicate scan page tuple byte views

## Scope

This packet covers the read-only HNSW duplicate-detection scan loops of Task 41 invariant #2 Phase D.

Code commit under review:

- `873a1fe0c843521089a5452bac42f9be668e6fe8` - `Scope HNSW duplicate scan page tuple byte views`

Changed file:

- `src/am/ec_hnsw/insert.rs`

## Change Summary

- Routed TurboQuant, TurboQuant V3, and grouped duplicate-detection scan loops through `shared::with_page_line_tuple_bytes`.
- Kept duplicate checks returning owned `ItemPointer` values only.
- Left duplicate coalescing rewrite paths for a separate slice because those require mutation/copy handling.

## Lifetime Argument

The duplicate scan loops now create page tuple byte slices only through the shared helper and consume them inside callbacks while each data block's buffer guard remains live. The callbacks decode owned tuple structs and return only optional TIDs.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head-insert.log`: `git diff --check HEAD -- src/am/ec_hnsw/insert.rs`, exit 0.

`git diff --check` was scoped to the file under review because unrelated comparator benchmark files were already dirty in the worktree.

## Review Notes

Please focus on whether each duplicate scan preserves the previous tag/deleted/heaptid/rerank matching behavior.
