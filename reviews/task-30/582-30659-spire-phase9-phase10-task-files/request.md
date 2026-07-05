# Review Request: SPIRE Phase 9/10 Task File Split

## Summary

Task 30 now has separate detailed task files for the expanded Phase 9 and
Phase 10 work:

- `plan/tasks/task30-phase9-spire-graph-architecture.md`
- `plan/tasks/task30-phase10-spire-execution-performance.md`

Planning checkpoint: `7868074c` (`Split SPIRE phase 9 and 10 task plans`)

## Scope

- Adds the requested `task30-phase9-...` style task file for graph
  architecture:
  - top-graph frontier contract;
  - scalable top-graph storage;
  - cached/borrowed graph routing;
  - global recursive beam and route budgets;
  - boundary replication execution contract;
  - global vector identity;
  - lower-priority quality experiments.
- Adds the matching Phase 10 execution/performance task file:
  - bounded candidate collection;
  - eager-vs-streaming AM scan decision;
  - heap rerank I/O;
  - multi-NVMe read overlap;
  - remote libpq executor;
  - remote heap resolution;
  - performance harness.
- Updates `plan/tasks/30-spire-ivf-foundation.md` so the main Task 30 plan is
  now the overview/index for Phases 9 and 10 instead of carrying all detail
  inline.
- Updates `plan/tasks/README.md` to list the two new phase task files under
  Task 30.

## Validation

- `git diff --check HEAD~1..HEAD`

## Notes

This is a planning-only checkpoint. It does not implement graph, scan, storage,
or libpq behavior. Product-scale performance claims remain gated on packet-local
measurement artifacts and the Phase 8 scale-packet policy.
