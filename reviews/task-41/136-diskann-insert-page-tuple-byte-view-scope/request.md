# Task 41 Invariant 2 Review Request: DiskANN insert page tuple byte views

## Scope

This packet covers the DiskANN insert/backlink page tuple slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `ebf49dc3a7c92df0321889a4d26c0644693f7925` - `Scope DiskANN insert page tuple byte views`

Changed file:

- `src/am/ec_diskann/insert.rs`

## Change Summary

- Added a local `with_page_tuple_bytes` helper around `page_tuple_location`.
- Routed duplicate-bind patch application, backlink insertion, and backlink mutation rewrites through the helper.
- Replaced duplicate-bind loop `continue` / retry `break` behavior inside the callback with `DuplicateBindApplyOutcome`, preserving the previous no-change/changed/retry handling outside the callback.

## Lifetime Argument

The DiskANN insert tuple byte slices are now created in one helper and decoded/copied inside callbacks while the registered WAL page remains in scope. Mutation paths no longer construct ad hoc `&[u8]` views at each call site; the encoded replacement copy stays inside the same callback invocation that validated tuple length.

The metadata special-page byte views in this file are unchanged and remain synchronous fixed-size metadata decodes under a metadata buffer guard.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the `DuplicateBindApplyOutcome` translation preserves the previous skip/change/retry loop behavior and whether the backlink tuple byte views are now sufficiently scoped to the current WAL page callback.
