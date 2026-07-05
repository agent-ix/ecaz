# Task 39 SpireLocalObjectStore edge cases

## Summary

Closes two specific gaps in
`src/am/ec_spire/storage/local_store.rs`:

1. `SpireLocalObjectStore::new(valid_relid, 0)` — the `page_size == 0`
   branch in `new_for_store` (line 38). The pre-existing test pinned
   `store_relid == 0` via `with_default_page_size(0)` but never hit
   the second guard.
2. `store.insert_top_graph_object(0, ...)` — the `epoch == 0` guard
   was already pinned for leaf / delta / routing inserts; top-graph
   was the missing third path through the same guard.

Both extensions live inside the existing
`local_object_store_rejects_invalid_store_and_epoch` test so the
related guards stay grouped.

## Code under review

- Commit: `7e589803f97636a53e0e2dfe0ee594757a9d3a73`
- Changed file: `src/am/ec_spire/storage/tests/local_store.rs`

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib
  local_object_store_rejects_invalid_store_and_epoch`: passed.
  Artifact: `artifacts/local-store-focused-tests.log`.
- Full storage test suite (74 tests) passes.

## Notes

- The new asserts are MIRI-safe (no `unsafe`, no pgrx interaction)
  so they pick up the hardening MIRI lane automatically.
- Remaining `local_store.rs` gaps (read_top_graph_object mismatched
  placement family; SpireLocalObjectStoreSet routing edge cases) are
  separate follow-up slices.
