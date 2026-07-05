# Review Request: SPIRE Mirror Sync ADR

## Summary

This packet adds ADR-066, the Stage D mechanism choice requested by reviewer
direction packet `30800`.

ADR-066 selects an explicit operator-owned mirror refresh mechanism for v1:
a SQL primitive wrapped by `ecaz`, run outside AM scans after remote endpoint
readiness and before an epoch is advertised as AM-deliverable. It rejects
background-worker-first, lazy per-query materialization, generic logical
replication first, and user-managed register calls as the primary production
mechanism.

The task file now breaks the mirror sync work into implementation slices:
profile/dry-run diagnostics, refresh SQL primitive, `ecaz` wrapper, no-explicit-
register PG18 fixture, and catalog lifecycle coverage.

## Files

- `spec/adr/ADR-066-spire-operator-owned-row-materialization-mirror-sync.md`
- `spec/adr/index.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local manifest is under `artifacts/manifest.md`.

- `git diff --check -- spec/adr/ADR-066-spire-operator-owned-row-materialization-mirror-sync.md spec/adr/index.md plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Reviewer Focus

- Confirm explicit operator-owned refresh is the right v1 mechanism before
  implementation starts.
- Confirm ADR-066’s rejected alternatives are accurately scoped.
- Confirm the implementation slice order matches the Stage D finish direction
  from packet `30800`.
