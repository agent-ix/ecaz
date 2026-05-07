# Review Request: SPIRE Recursive Routing Support Diagnostic

Head SHA: `4e032fdf`

## Summary

`ec_spire_index_hierarchy_snapshot(index_oid)` now reports recursive routing as
supported when the active hierarchy is valid and contains internal routing
objects.

Single-level indexes still report `recursive_routing_supported = false`.
Malformed recursive shapes also remain unsupported because the support flag is
gated on the existing recursive hierarchy shape validator.

The existing PG18 recursive fanout build smoke now asserts the support flag turns
on for an opt-in recursive build.

## Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 27 passed, including PG18 pg-test
    `pg_test_ec_spire_recursive_fanout_build_hierarchy`.
- `git diff --check`

## Review Focus

- Confirm the support predicate should be `hierarchy_shape_valid &&
  internal_routing_object_count > 0`.
- Confirm single-level indexes should continue reporting unsupported, even
  though they remain queryable through the single-level path.
- Confirm the updated status recommendations match the current Phase 3 state.
