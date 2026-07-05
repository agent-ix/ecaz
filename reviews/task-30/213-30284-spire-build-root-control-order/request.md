# SPIRE Build Root Control Order

## Checkpoint

- Code commit: `e3f8f567` (`Initialize SPIRE root control before build scan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: reviewer F7 preparatory ordering for `ambuild`

## Summary

This checkpoint moves empty root/control initialization before `table_index_build_scan` in `ec_spire_ambuild`. The current callback still rejects populated builds, but future populated callbacks now start from an index relation with block 0 initialized before any relation-backed object writes can happen.

This is not the full populated-build publish path. It is a narrow ordering fix that keeps the empty-build behavior intact and aligns with the object tuple guard that requires root/control to exist before appending object tuples.

## Changed Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_empty_build_scan_no_rows --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1068 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `188 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- Full F7 remains the populated `ambuild` implementation: collect rows, write objects, write placements, write manifests, then publish active root/control through the coordinator.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
