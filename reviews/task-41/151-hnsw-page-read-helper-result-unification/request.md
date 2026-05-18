# Task 41 invariant #2 review request: HNSW page read helper result unification

## Scope

This packet covers code commit `98449e8f28a0fd5d685438767ed676477ce7cdff`
(`Unify HNSW page tuple read helper results`).

The slice handles packet 147 seq 2 items D and F for the HNSW read helper
surface:

- `shared::with_page_line_tuple_bytes` now returns
  `Result<Option<R>, String>` instead of calling `pgrx::error!` internally for
  invalid tuple bounds.
- HNSW callers map `Err` to their existing `pgrx::error!` boundary and keep
  `None` as the unused-line-pointer result.
- `scan_debug` now delegates its debug helper to the shared helper instead of
  duplicating line-pointer and tuple-bound logic.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check 98449e8f^ 98449e8f` passed.

## Reviewer Focus

- Confirm invalid tuple bounds are now surfaced as `Err(String)` from the
  shared helper rather than an internal `pgrx::error!`.
- Confirm unused line pointers still map to `Ok(None)`.
- Confirm `scan_debug` uses the shared helper while preserving its debug skip
  behavior for helper errors.
