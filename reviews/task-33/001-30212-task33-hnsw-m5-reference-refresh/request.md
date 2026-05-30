# Task 33 HNSW M5 Reference Refresh Scaffold

Reviewer: please review this Task 33 measurement scaffold before the long M5
worker sweep is run.

## Scope

This starts Task 33 with a suite-runner-owned measurement plan for the required
HNSW M5 reference refresh. It does not change runtime code and does not make
new benchmark claims.

Added:

- `task33-hnsw-m5-reference-refresh.packet.json`
- `artifacts/task33_hnsw_m5_worker_sweep.sql`
- `artifacts/manifest.md`

The suite first uses `ecaz bench suite` to load the real50K HNSW fixture, then
uses a `raw` suite step to execute the worker-sweep SQL through `ecaz dev sql`.
The SQL records worker-process headroom, builds `ec_hnsw` source-scored indexes
with requested worker counts `1/2/4/8`, captures
`tests.ec_hnsw_debug_last_build_timing()`, and keeps the `w4` index under the
loader's canonical `task33_m5_hnsw_real50k_m16_idx` name for follow-on
recall/latency/storage suite steps.

## Why This Shape

Task 33 needs `ALTER TABLE ... SET (parallel_workers = N)` and session GUCs for
each build. The suite runner does not have a first-class worker-sweep step yet,
so this packet keeps the matrix inside `ecaz bench suite` with a raw
packet-local SQL step instead of adding a bash sweeper.

The SQL follows the Task 26 worker-sweep shape but uses the Task 33 prefix and
the checked-in M5 real50K staged corpus.

## Validation

- `ecaz bench suite audit --config reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/task33-hnsw-m5-reference-refresh.packet.json`
- `ecaz bench suite run --dry-run --config reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/task33-hnsw-m5-reference-refresh.packet.json --manifest-output reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/artifacts/dry-run-suite-manifest.json`
- `git diff --check`

No benchmark run was executed in this checkpoint; the expected worker sweep is
long-running and should be launched as the next packet after this scaffold is
accepted.

## Review Focus

- Does the packet preserve Task 33's required worker-count/headroom evidence?
- Is the `raw` suite step acceptable for the GUC/table-option sweep, or should
  the next checkpoint add a first-class suite step type before running it?
- Is selecting the `w4` index for recall/latency/storage reasonable as a first
  M5 refresh, given Task 26's previous best point?
