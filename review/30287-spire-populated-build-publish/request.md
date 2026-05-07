# SPIRE Populated Build Publish

## Checkpoint

- Code commit: `63e1dcaf` (`Publish SPIRE populated build epoch`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: relation-backed populated `ambuild` publication

## Summary

This checkpoint wires populated `ec_spire_ambuild` through the initial strict local epoch publish path:

- The build callback collects heap rows into SPIRE assignment inputs and source vectors.
- The build state trains a single-level centroid plan using `nlists`, `training_sample_rows`, and `seed`.
- The relation object store writes one root routing object plus one segmented V2 leaf object per centroid, preserving empty centroid leaves.
- Placement entries are written as durable relation tuples and the object manifest records those placement-entry locators.
- Manifest bundles are written to relation tuples, then root/control is advanced to active epoch 1 with next PID and local vec-id cursors.
- A new PG18 test verifies that populated `CREATE INDEX ... USING ec_spire` succeeds and publishes root/control.

The active-epoch scan path remains intentionally open; `amrescan` still needs relation-backed snapshot loading before populated SPIRE indexes can serve queries.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_populated_build_publishes_root_control --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1076 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `196 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- PQ-FastScan populated builds remain blocked by the existing assignment encoder until persisted grouped-PQ model metadata lands.
- This checkpoint does not claim recall or latency behavior.
- No measurement artifacts are included; validation is functional PG18 coverage only.
