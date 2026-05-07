# SPIRE Pinned Relation Tuple Reads

## Checkpoint

- Code commit: `743a1dff` (`Add SPIRE pinned relation tuple reads`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: reviewer F2 follow-up for relation-backed object tuple reads

## Summary

This checkpoint adds `page::with_pinned_object_tuple`, a closure-based object tuple reader that keeps the buffer pinned and locked while the caller decodes a borrowed `&[u8]`. The existing owned `read_object_tuple` path remains available by cloning inside that helper.

`SpireRelationObjectStore` now uses the pinned helper for:

- object header dispatch,
- routing / leaf / delta single-tuple object decode,
- V2 leaf meta decode,
- V2 leaf segment decode.

The current `SpireObjectReader` trait still returns owned object structs, so V2 leaf segments are still materialized before returning. This nevertheless removes the extra relation-tuple `Vec<u8>` allocation at the page-cache boundary and leaves the pinned API in place for future scan callbacks that can consume column views while the page is pinned.

## Changed Files

- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/storage.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_relation_object_tuple_roundtrip --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1068 filtered out`
- `cargo test --lib test_ec_spire_relation_leaf_v2_roundtrip --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1068 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `188 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This addresses the API portion of reviewer F2. A future scan-loading slice can add a callback/visitor that consumes V2 column segments directly under each segment pin instead of returning an owned `SpireLeafPartitionObjectV2`.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
