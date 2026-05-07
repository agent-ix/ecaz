# SPIRE Active Snapshot Diagnostics

## Checkpoint

- Code commit: `2466e11e` (`Add SPIRE active snapshot diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: relation-backed active snapshot diagnostics

## Summary

This checkpoint exposes the existing SPIRE snapshot diagnostics over the
persisted relation-backed active epoch for test/debug validation:

- `load_relation_epoch_manifests` is now visible within `ec_spire` so debug
  helpers can share the same manifest decode and validation path as live scans.
- `debug_spire_active_snapshot_diagnostics` opens an `ec_spire` index relation
  read-only, reads root/control manifest locators, decodes the active epoch,
  constructs a published snapshot, and reads partition objects through
  `SpireRelationObjectStore`.
- The populated-build PG18 test now asserts active snapshot cardinality:
  three objects, three placements, one local store, one root object, two leaf
  objects, two routing children, three leaf assignments, and nonzero available
  object bytes.
- The task plan now records relation-backed snapshot diagnostics as covered for
  active strict local epochs.

This remains an internal test/debug surface. SQL/admin exposure and
quantizer/build-parameter reporting are still open.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_populated_build_publishes_root_control --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1076 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `196 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Insert-after-build, delete/vacuum cleanup, exact heap rerank, PQ-FastScan
  scorer binding, and SQL/admin diagnostics remain open.
