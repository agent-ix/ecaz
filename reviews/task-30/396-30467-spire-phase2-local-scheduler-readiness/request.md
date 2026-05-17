# Review Request: SPIRE Phase 2 Local Scheduler Readiness

## Summary

Task 30 SPIRE Phase 2 local single-store manual scheduler scope is ready for
branch-local landing review.

Completed local scheduler scope:
- Replacement leaf planning, routing rewrite, placement directory, object
  write, draft assembly, relation publish, and publish-lock planning helpers.
- Selected split/merge execution input builders for local dry-runs and
  relation-backed publish.
- Heap-source split materialization that omits dead heap rows before validating
  exact live-source coverage.
- Live manual `ec_spire_index_maintenance_run(index_oid)` entrypoint that takes
  the publish lock, reloads/rechecks the selected candidate, writes replacement
  objects, publishes the successor epoch, and returns a run-result row.
- Locked no-write `ec_spire_index_locked_maintenance_run_plan(index_oid)`
  preflight that projects the same selected publish plan without mutating the
  active epoch.
- SQL volatility marking for the mutating maintenance entrypoint.
- Focused PG18 SQL smoke coverage for empty, populated no-candidate, locked
  no-write, locked-plan-to-live-publish consistency, merge publish, merge rerun
  no-op, split publish, and post-publish scan visibility.

Deferred later-phase scope:
- Background maintenance scheduling around the manual entrypoint.
- Old-epoch physical reclamation after retention and active-query safety rules.
- Longer mixed insert/delete/scan stress beyond the focused same-leaf insert
  serialization coverage already present.
- Recursive hierarchy, multi-store placement, boundary replication, top-level
  graph routing, remote placement, and product-scale measurement evidence.

## Validation

- `cargo pgrx test pg18 maintenance_run`
- `git diff --check`

## Notes

No measurement claims.
