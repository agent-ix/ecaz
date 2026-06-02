# Artifact Manifest: Task 71 Phase 1 IVF parallel build design

- head SHA: c72fe9903342ced3d4d306e1f952c4a2847427f7
- task bucket: `reviews/task-71/001-phase1-design/`
- lane: design decision
- fixture: none
- storage format: IVF build path, all current formats considered at design level
- rerank mode: not applicable
- command used: static review of repository files
- timestamp: 2026-06-02
- isolated one-index-per-table or shared-table surface: not applicable

## Sources Reviewed

- `plan/tasks/71-ivf-parallel-build.md`
- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/routine.rs`
- `src/am/ec_hnsw/build_parallel.rs`
- `spec/adr/ADR-048-parallel-hnsw-build-graph-assembly.md`
- `plan/tasks/26-parallel-index-build.md`
- `reviews/task-69/003-training-parallelism-measurement/`
- `reviews/task-69/004-closeout/`

## Key Result Lines

- Decision: choose Option A, per-worker heap tuple collection with leader-side training, assignment, staging, and flush.
- Determinism contract: leader sorts merged worker tuples by heap TID before training/staging.
- Fallback contract: no-worker or failed parallel setup path must return to the existing serial build path before flushing index pages.
- Validation: no tests run; design-only checkpoint.
