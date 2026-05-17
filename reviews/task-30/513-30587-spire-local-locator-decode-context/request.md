# SPIRE Local Locator Decode Context

## Summary

This packet addresses the 30579 feedback on local heap locator decode errors.

`remote_search_local_heap_resolution_plan_rows(...)` and
`remote_search_local_heap_candidate_rows(...)` now share
`decode_remote_search_local_heap_locator(...)`, which wraps
`ItemPointer::decode` failures with candidate context:

- caller context
- `pid`
- `row_index`
- hex-encoded `vec_id`

The successful decode path and SQL-visible row values are unchanged.

## Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/tests.rs`

## Validation

Head SHA: `fad793c9`

- `cargo test --lib remote_local_heap_locator_decode_error_includes_candidate_context --no-default-features --features pg18`
- `cargo check --lib --no-default-features --features pg18`

Result:

- Focused Rust unit test passed: 1 passed; 0 failed; 1442 filtered out.
- PG18 lib check passed.

## Notes

This stays deliberately local to error reporting. It does not change remote
candidate validation or the row-locator encoding contract.
