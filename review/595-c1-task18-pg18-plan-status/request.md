# Task 18 PG18 Plan Status Alignment

## Summary

This packet covers commit `6969dae807c92d1f4fcbdf85e0fd2acd3f61bf63`.

`plan/tasks/18-parallel-index-scan.md` now reflects the current PG18 state:

- planner-visible `Parallel Index Scan` activation has landed for PG18 `ec_hnsw`;
- the production path uses one elected tuple emitter for serial-equivalent output;
- direct multi-emitter output remains diagnostic-only because it is not yet compatible with `Gather Merge` ordering;
- PG18 is the default Task 18 validation lane, while PG17 is retained as compatibility coverage.

The update also checks off the PG18 callback-wiring subtasks that are now live and replaces stale notes that still described `amcanparallel=false`.

## Review Focus

- Confirm the plan accurately distinguishes shipped PG18 elected-emitter behavior from future multi-emitter latency work.
- Confirm the checked callback subtasks match the current implementation scope.
- Confirm the references to review packets 593/594 and ADR-040 are enough to support the `Gather Merge` blocker summary.

## Artifacts

- `artifacts/manifest.md`

This is a docs-only packet. Measurement logs cited by the task-plan update remain in packets 590, 593, and 594 rather than being duplicated here.

## Validation

- `git diff --check`
