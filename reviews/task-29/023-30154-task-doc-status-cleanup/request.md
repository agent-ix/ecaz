# Task Docs and README Status Cleanup

## Scope

Docs-only cleanup after the Task 28 IVF and Task 29 DiskANN lanes landed on
`main`.

## Changes

- Updated the root `README.md` to describe the current access-method set:
  `ec_hnsw`, `ec_ivf`, and `ec_diskann`.
- Updated `docs/architecture.md` and `docs/pg18.md` so they no longer describe
  the project as HNSW-only or imply parallel index scan is an active PG18
  callback follow-up.
- Updated planning docs to mark:
  - Task 29 / 29a / 29b / 29c / 29d as landed on `main`.
  - Task 29e as cleanup/evidence, not a blocker.
  - Task 28 IVF access method and competitive substrate as landed for the local
    v1 lane, with product-scale benchmarking deferred.
  - Task 19 parallel scan callbacks as shelved with Task 18.
  - Task 26 HNSW parallel build as landed for local PG18 builds, with larger
    scale work deferred to AWS/RDS-class hardware.
- Updated `plan/tasks/README.md` so the task-board summary matches the current
  landed/deferred state.

## Validation

- `git diff --check`
- Stale-status scan over `README.md`, `docs/`, and `plan/tasks/` for:
  `ready for review`, `ready for round`, `landed on branch`,
  `merge of task28-ivf`, `branch is not merge-ready`,
  `optional parallel-scan`, `Phase 8 measurement gates next`,
  `Task 29 landing blocker`, and `ready for merge review`

## Artifacts

No measurement artifacts. This is a docs-only status cleanup.
