# 30362 SPIRE Replacement Leaf Planning — feedback

Read `src/am/ec_spire/update.rs` and `plan/design/spire-update-mechanics.md`.

## What landed

`plan_leaf_replacement_pids` allocates replacement leaf PIDs from a snapshot
of the root/control PID allocator cursor. Split allocates ≥2 fresh PIDs,
merge allocates 1 fresh PID, rebalance reuses the existing PID only when
`parent_centroid_byte_equal`. A row-folding helper reads the active epoch
snapshot, folds insert/delete deltas into base-leaf rows, clears the
delta-insert flag on survivors, and fails closed when a target PID isn't an
active leaf.

## Correctness

- Allocator-cursor handling is right: the helper observes the affected PIDs
  on a *copy* of the cursor before calling `allocate()`, then commits the
  cursor only on the success path (line 411). A failure mid-plan does not
  leak allocator advances.
- Rebalance's PID-reuse gate (`parent_centroid_byte_equal` required) matches
  the design doc invariant that "coverage does not change" means byte-equal
  centroid in the parent.
- `affected_leaf_pids` validation (non-empty, unique, non-zero) is
  centralized in `validate_affected_leaf_pids`, reused by every downstream
  helper.

## Status

Lands cleanly. This is the right primitive — every later scheduled-publish
helper composes on top of the `SpireLeafReplacementPidPlan` shape it
produces.
