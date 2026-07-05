# SPIRE Allocator Diagnostics

## Checkpoint

- Code commit: `9902261f`
  (`Expose SPIRE allocator diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for root/control PID and local vec-id allocation
  cursors

## Summary

This checkpoint exposes the active SPIRE allocator state directly:

- Added `ec_spire_index_allocator_snapshot(index_oid, warn_within)` as a
  stable, strict SQL table function.
- The function reports active epoch, caller-provided warning threshold, next
  PID, remaining PID allocations, PID near-exhaustion flag, next local vec-id
  sequence, remaining local vec-id allocations, and local vec-id
  near-exhaustion flag.
- Remaining allocation counts are returned as text because normal healthy
  values can exceed PostgreSQL `bigint`.
- Negative warning thresholds are rejected before opening the index relation.
- The focused SQL test verifies an empty index starts with epoch `0`, next PID
  `1`, next local vec-id sequence `1`, and full remaining PID headroom; after
  the first insert bootstraps the index it verifies epoch `1`, next PID `3`,
  next local vec-id sequence `2`, and both near-exhaustion flags remain false.
- Updated the Task 30 plan to record this allocator surface alongside the
  existing active snapshot, options, health, placement, object, hierarchy, and
  delta diagnostics.

This is read-only observability. It does not change PID allocation, local
vec-id allocation, publish semantics, placement, persistence, or cleanup.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_allocator_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `223 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- The diagnostic reads the root/control page and delegates near-exhaustion
  arithmetic to the existing allocator diagnostics helper.
- Replicas and remote placement remain deferred; this surface only reports the
  local root/control allocator state for the active index relation.
