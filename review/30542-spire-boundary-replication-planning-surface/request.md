# Review Request: SPIRE Boundary Replication Planning Surface

- Branch: `main`
- Code commit: `26457b68` (`Add SPIRE boundary replication planning surface`)
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Scope: related first implementation slices after the Phase 5 design
  checkpoint

## Summary

This checkpoint adds the default-off boundary-replication planning surface
without writing replica assignment rows yet.

It:

- adds the bounded `boundary_replica_count` `ec_spire` reloption
  (`0..=8`, default `0`);
- exposes `boundary_replica_count`, `boundary_replication_enabled`, and
  `scan_dedupe_mode` in `ec_spire_index_options_snapshot`;
- switches resolved scan plans to `VecIdDedupeEnabled` when the index is
  replica-capable, while preserving the current no-HashMap primary-only scan
  plan for the default;
- registers `boundary_replica_count` as a known `ecaz-cli` SPIRE profile
  reloption;
- adds a pure route-map helper that resolves one primary leaf PID plus bounded
  secondary replica leaf PIDs using existing top-N route ordering;
- keeps route tie-breaks deterministic: higher inner product, lower centroid
  ordinal, lower child PID;
- documents the new diagnostic columns and records the Phase 5 tracker status.

This intentionally stops before assignment fanout writes. Existing indexes keep
primary-only assignment rows unless a later checkpoint writes
`BOUNDARY_REPLICA` rows.

## Files

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/build/routing_plan.rs`
- `src/am/ec_spire/root/{types,snapshots}.rs`
- `src/lib.rs`
- `crates/ecaz-cli/src/profiles.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`
- focused tests in `src/am/ec_spire/build/tests/centroid_state.rs`

## Review Focus

1. Confirm deriving scan dedupe mode from `boundary_replica_count > 0` is
   acceptable for this planning slice.
2. Confirm the pure route-map helper is the right place to centralize
   primary-plus-secondary leaf PID selection before build/insert fanout writes.
3. Check that the SQL options snapshot columns are sufficient for operators to
   see whether an index is replica-capable.
4. Confirm the default path still avoids vec-id dedupe allocation.
5. Confirm this checkpoint is correctly scoped before writing
   `BOUNDARY_REPLICA` rows.

## Validation

- `cargo test --lib boundary_replica --no-default-features --features pg18`
- `cargo test --lib replica --no-default-features --features pg18`
- `cargo test -p ecaz-cli profiles`
- `cargo pgrx test pg18 test_ec_spire_options_snapshot_sql`
- `git diff --check`
