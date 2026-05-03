# SPIRE Populated Build Tuple Collection

## Checkpoint

- Code commit: `a90a66ca` (`Collect SPIRE populated build tuples`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: first populated-`ambuild` slice for reviewer F7

## Summary

This checkpoint changes `ec_spire_ambuild` from a callback that immediately rejects every populated row into a real source-tuple collection pass. The build callback now:

- Resolves the indexed heap column as `ecvector` or `tqvector`, matching the existing single-column AM restriction.
- Decodes heap TIDs from PostgreSQL item pointers.
- Detoasts the indexed datum and collects a source vector.
- Encodes SPIRE assignment inputs with the selected assignment payload format.
- Validates non-null input, finite non-zero vectors, consistent dimensions, heap-TID identity, and payload-format consistency.
- Preserves the current persistence boundary by failing after collection for nonempty builds with an explicit "publish is not implemented yet" error.

This keeps the persisted write/publish sequence out of this slice. It prepares the next slice to train centroids and publish relation-backed routing/leaf objects through the coordinator.

## Changed Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib build_state --no-default-features --features pg18`
  - `14 passed; 0 failed; 0 ignored; 0 measured; 1059 filtered out`
- `cargo test --lib test_ec_spire_empty_build_scan_no_rows --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1072 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `192 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is a staging checkpoint, not the full populated build path.
- The next implementation slice should decide the live relation-backed `placement_tid` semantics for the manifest bundle before publishing a populated epoch.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
