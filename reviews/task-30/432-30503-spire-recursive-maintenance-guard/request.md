# Review Request: SPIRE Recursive Maintenance Guard

Head SHA: `6716d198`

## Summary

SPIRE maintenance split/merge planning now fails closed for recursive
hierarchies until recursive update propagation lands. The guard inspects the
active snapshot's available object headers and rejects either:

- an internal routing object; or
- a root routing object above level 1.

The guard runs before leaf snapshot collection and replacement selection for:

- `ec_spire_index_maintenance_plan_snapshot`;
- `ec_spire_index_locked_maintenance_run_plan`;
- `ec_spire_index_maintenance_run`.

This prevents the Phase 2 replacement machinery from rewriting a level-1
parent while leaving higher recursive routing levels stale.

## Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`

## Validation

- `cargo test recursive_maintenance_guard_rejects_recursive_hierarchy -- --nocapture`
  - 1 passed: `recursive_maintenance_guard_rejects_recursive_hierarchy`.
- `cargo test recursive_maintenance_run_rejected -- --nocapture`
  - 1 passed: `pg_test_ec_spire_recursive_maintenance_run_rejected`.
  - PostgreSQL raised
    `ec_spire maintenance split/merge is deferred for recursive SPIRE indexes until recursive update propagation lands`
    through `ec_spire_index_maintenance_run`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm the guard belongs at the SQL maintenance entrypoints rather than
  inside the lower-level replacement planner.
- Confirm root level > 1 or any internal routing object is the right recursive
  hierarchy predicate.
- Confirm failing closed is preferable to returning `no_action` for recursive
  indexes during the Phase 3 to Phase 4 gap.
