# Review Request: SPIRE Boundary Replica Placement Diagnostics

## Summary

Closes the Phase 12.7 row:

> Add operator diagnostics for stale, missing, or unavailable boundary replica
> placements and their degraded-mode reporting.

This adds `ec_spire_index_boundary_replica_placement_diagnostics(index_oid)`,
an operator/debug SQL surface that groups placement health by global `vec_id`.
It reports primary assignment coverage, boundary-replica assignment coverage,
stale/unavailable/skipped replica placement counts, node span, status, degraded
mode action, and recommendation.

The PG18 fixture covers:

- missing boundary replica assignment coverage on an index without boundary
  replicas;
- unavailable boundary replica placement state with `skip_and_report`;
- skipped boundary replica placement state with `skip_and_report`; and
- stale boundary replica placement state with `fail_closed`.

The test-only placement rewrite helper now decodes manifests permissively enough
to rewrite an already invalid placement state into the next invalid state needed
by the fixture.

## Files

- `src/am/ec_spire/root/diagnostics.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `git diff --check 5437395e^ 5437395e`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_placement_diagnostics_sql`

## Reviewer Focus

- Confirm the diagnostic can inspect stale, unavailable, and skipped placement
  states without weakening the strict published-snapshot checks used by normal
  scan/read paths.
- Confirm grouping by global `vec_id` and reporting missing/stale/unavailable/
  skipped boundary replica conditions is sufficient operator coverage for this
  Phase 12.7 tracker row.
- Confirm the degraded-mode actions are conservative: stale/missing fail closed,
  unavailable/skipped report as `skip_and_report`.
