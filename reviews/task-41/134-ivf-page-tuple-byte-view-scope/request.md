# Task 41 Invariant 2 Review Request: IVF page tuple byte views

## Scope

This packet covers the IVF page slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `9a3e137128b7848ace8ec5b223699a8506753512` - `Scope IVF page tuple byte views`

Changed file:

- `src/am/ec_ivf/page.rs`

## Change Summary

- Added local `with_page_line_tuple_bytes` / `with_required_page_tuple_bytes` helpers in `src/am/ec_ivf/page.rs`.
- Routed IVF posting iteration, posting-ref iteration, directory tuple rewrite decode, debug posting block summary, posting rewrite decode, generic tuple read, and tag search through the closure helpers.
- Left metadata special-area byte views unchanged because they are fixed-size metadata special-page reads/writes under an active metadata buffer or registered WAL page, not line-pointer tuple views.

## Lifetime Argument

The helper creates the raw page tuple byte slice and immediately invokes a callback. Callers decode to owned values or use the borrowed `IvfPostingTupleRef` only inside that callback. The only raw tuple slice construction that remains in `page.rs` is inside the helper itself; the other remaining `from_raw_parts` sites are metadata special-page views.

This preserves the existing buffer/WAL ownership boundaries:

- Shared-read visitors call the helper while a `LockedBufferGuard` is live.
- Rewrite paths call the helper while the exclusive buffer and `GenericXLogTxn` registered page are live.
- Required tuple reads still validate the caller's offset range before invoking the helper.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the helper closure boundary is tight enough for the IVF page tuple views and whether the WAL rewrite paths still preserve the prior decode/rewrite behavior.
