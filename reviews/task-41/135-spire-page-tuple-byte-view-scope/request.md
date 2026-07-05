# Task 41 Invariant 2 Review Request: SPIRE page tuple byte views

## Scope

This packet covers the SPIRE page slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `e7cfe783c5b0e32799454ab11f1c44d3d82d5164` - `Scope SPIRE page tuple byte views`

Changed file:

- `src/am/ec_spire/page.rs`

## Change Summary

- Added a local `SpireObjectTupleVisit` result so object tuple scans can treat unused slots as non-errors while required tuple reads still report them.
- Collapsed object tuple byte-slice construction into the existing local callback helper path.
- Changed `rewrite_object_tuple_same_len` so the same-length payload copy happens inside the object tuple callback instead of returning a tuple pointer out of the callback.

## Lifetime Argument

The SPIRE object tuple byte slice is now constructed in one helper and consumed by callbacks while the caller's buffer page is still locked. The rewrite path no longer lets a tuple pointer escape the callback boundary; validation and copy are both performed inside the callback before the page guard/WAL transaction can move on.

The root/control metadata slice remains a fixed-size special-page view decoded synchronously under the metadata buffer guard.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the rewrite copy is now correctly scoped inside the callback and whether the scan behavior for unused line pointers is unchanged.
