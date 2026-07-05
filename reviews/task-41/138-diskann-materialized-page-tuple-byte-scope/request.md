# Task 41 Invariant 2 Review Request: DiskANN materialized page tuple bytes

## Scope

This packet covers the DiskANN scan-state materialization tuple-copy slice of Task 41 invariant #2 Phase D.

Code commit under review:

- `a75bcc8379567a6902080b74eb6682779807715f` - `Scope DiskANN materialized page tuple bytes`

Changed file:

- `src/am/ec_diskann/scan_state.rs`

## Change Summary

- Added local `copy_data_page_tuple_bytes` for data-page tuple materialization.
- Moved tuple pointer lookup, bounds validation, byte-slice construction, and owned `Vec<u8>` copy into the helper.
- Kept `DataPageChain::insert_raw_tuple` receiving owned bytes only.

## Lifetime Argument

The data-page tuple borrow is now confined to `copy_data_page_tuple_bytes`, where it is immediately copied to an owned `Vec<u8>` while the `LockedBufferGuard` for the current block is live. No borrowed page slice is returned to the caller.

The metadata special-page view in this function remains a fixed-size metadata decode under the metadata buffer guard.

## Validation

Artifacts are under `artifacts/`:

- `cargo-fmt-check.log`: `cargo fmt --all --check`, exit 0.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`, exit 0 with the pre-existing `src/am/mod.rs` unused-import warning.
- `git-diff-check-head.log`: `git diff --check HEAD`, exit 0.

## Review Notes

Please focus on whether the new helper preserves materialization behavior while making the page-borrow boundary explicit.
