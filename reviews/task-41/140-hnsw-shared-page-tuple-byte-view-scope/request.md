# Task 41 Invariant 2 Review Request: HNSW shared page tuple byte views

## Scope

This packet covers the HNSW shared page utility slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `b42091aec8942f50c3e7c81ad2f94daf8886fce9` - `Scope HNSW shared page tuple byte views`

Changed file:

- `src/am/ec_hnsw/shared.rs`

## Change Summary

- Added private `with_page_line_tuple_bytes` for page line-pointer byte views.
- Routed live tuple counting, highest-level live entry selection, and debug data-page reads through the helper.
- Kept metadata full-page decode views unchanged; they remain synchronous metadata page decodes under active buffer guards.

## Lifetime Argument

The helper creates tuple byte slices from a page line pointer and invokes a closure immediately while the caller's `LockedBufferGuard` is still live. Count/selection paths consume decoded values in-place, and the debug read path copies tuple bytes into owned `Vec<u8>` before returning.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether count/entry/debug behavior is preserved and whether the helper keeps tuple byte views scoped to the active data-page buffer.
