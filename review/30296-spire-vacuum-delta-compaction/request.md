# SPIRE Vacuum Delta Compaction

## Checkpoint

- Code commit: `9ec0aef5` (`Compact SPIRE vacuum delta epochs`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: strict local `amvacuumcleanup` compaction of active SPIRE delta
  objects into replacement V2 base leaf objects

## Summary

This checkpoint extends the populated relation-backed `ec_spire` vacuum path
after delete-delta publication:

- `amvacuumcleanup` now takes the same vacuum publish lock, loads the active
  root/control state, and checks the active object manifest for delta objects.
- When active deltas exist, cleanup collects visible assignments after applying
  delete-delta suppression, groups them by affected base leaf PID, clears the
  transient `DELTA_INSERT` assignment flag, and rewrites each affected leaf as a
  replacement V2 base leaf object.
- The replacement epoch carries root/internal/unaffected leaf placements
  forward, omits delta-object placements from the active directory, bumps the
  rewritten leaf object versions, persists a new manifest bundle, and advances
  root/control to the cleanup epoch without consuming new PIDs or local
  `vec_id`s.
- The focused vacuum test now verifies the delete-delta epoch followed by a
  cleanup epoch, confirms live tuple stats, checks root/control cursors, and
  uses SQL diagnostics to assert one active leaf assignment and zero active
  delta objects after cleanup.
- The Task 30 plan now records vacuum delta compaction as covered, while
  leaving physical page reclamation, old-epoch cleanup, and real SQL VACUUM
  end-to-end coverage open.

This does not implement physical page reclamation, retention-window cleanup of
old object tuples, real SQL `VACUUM` end-to-end validation, insert batching,
split/merge triggers, or PQ-FastScan scorer binding.

## Changed Files

- `src/am/ec_spire/vacuum.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_vacuum_delete_delta_suppresses_visible_row --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `202 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- The old immutable object tuples remain in relation pages for a later
  retention/cleanup pass; this checkpoint only removes delta placements from
  the newly published active directory.
