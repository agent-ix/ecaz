# Task 33 Offline Builder ADR

Reviewer: please review the Task 33 Phase 2 design-lane checkpoint.

## Scope

This packet adds `spec/adr/ADR-073-hnsw-staged-offline-bulk-build.md` and
updates `spec/adr/index.md`.

It is driven by the Task 33 measurement packets:

- `002-30213-task33-hnsw-m5-reference-refresh-run`
- `003-30214-task33-hnsw-m5-reference-refresh-100k`

Both runs show requested workers improve build wall time up to the 4-worker
surface and regress at 8 requested workers. That repeats the Task 26/ADR-048
shape closely enough to stop worker-threshold tuning.

## Decision

ADR-073 proposes moving HNSW follow-up to a staged/offline bulk-build lane while
keeping ADR-048 concurrent DSM graph assembly as the current in-PostgreSQL
fallback.

## Validation

- `git diff --check`

No tests or benchmarks were run for this docs-only design checkpoint.

