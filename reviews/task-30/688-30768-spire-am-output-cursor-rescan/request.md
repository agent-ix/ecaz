# 30768 - SPIRE AM Output Cursor Rescan Coverage

## Summary

This packet reviews commit `e3d2f2c069ff729771dcce4af74f8eb3158801f0`
(`Cover SPIRE AM output cursor rescan`).

The slice addresses the `30762` reviewer P3 on scan-opaque cursor
advance/exhaustion and stream-rebuild-on-rescan semantics.

It adds focused runtime-state coverage proving:

- default `SpireScanOpaque` has no output rows and has not been rescanned;
- a reset output cursor advances row-by-row, reports remaining rows, and stays
  exhausted once drained;
- a later `reset_for_outputs` call replaces the exhausted cursor and query state
  with the new rescan stream instead of reusing stale rows.

This is coverage only. It does not implement remote row materialization.

## Key Files

- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `git diff --check -- src/am/ec_spire/scan/tests/runtime_state.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo fmt --check`
- `cargo test scan_opaque_rescan_replaces_exhausted_output_cursor --no-default-features --features pg18`

No PostgreSQL distributed fixture or performance run was started for this
packet.

## Review Focus

- Does this cover the important AM cursor state transitions before the real
  materialized-row provider lands?
- Is `reset_for_outputs` clearly replacing prior stream state on every rescan?
- Is the Phase 11 note scoped correctly as coverage rather than implementation?
