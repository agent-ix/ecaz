# SPIRE Placement State Diagnostics Coverage

## Checkpoint

- Code commit: `1e5bc88e`
  (`Clarify SPIRE placement state diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30299`

## Summary

This checkpoint tightens store-placement diagnostic coverage around
non-available placement states.

The existing degraded-mode store-placement test now explicitly covers both
`Unavailable` and `Skipped` placements and verifies they contribute to
placement-state counters and total placement bytes without contributing to
available objects, assignment counts, or per-kind object bytes.

The `Stale` state is different: valid published snapshots reject stale
placements even in degraded mode. This checkpoint records that boundary with a
unit test and comments the stale branches as defensive code for future
retained-placement diagnostics.

## Changed Files

- `src/am/ec_spire/diagnostics.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test store_placement_diagnostics_ --no-default-features --features pg18`
  - `3 passed; 0 failed; 1121 filtered out`
- `git diff --check`

## Notes

- This responds to the packet `30299` review coverage gap while preserving the
  published-snapshot invariant that `Stale` is not a valid degraded placement
  state.
