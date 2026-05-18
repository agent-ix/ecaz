# Task 41 Invariant 2 Review Request: DiskANN vacuum page tuple byte views

## Scope

This packet covers the DiskANN vacuum/rewrite page tuple slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `2f11721a4c4df4b9a0f46250e389f2f3bca1a7a4` - `Scope DiskANN vacuum page tuple byte views`

Changed file:

- `src/am/ec_diskann/routine.rs`

## Change Summary

- Added local `with_vacuum_page_tuple_bytes` around `vacuum_page_tuple_location`.
- Routed vacuum expected-raw comparison, replacement copy, and pg-test raw tuple rewrite through the helper.
- Kept tuple length validation and replacement copy inside the helper callback.

## Lifetime Argument

DiskANN vacuum tuple bytes are now constructed in one helper and consumed inside callbacks while the relevant page remains pinned/locked. The read-compare path only returns a boolean outcome, and the write paths validate and copy replacement bytes inside the same callback invocation.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the retry comparison and replacement-copy behavior still matches the prior vacuum rewrite logic.
