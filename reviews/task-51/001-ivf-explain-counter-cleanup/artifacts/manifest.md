# Artifact Manifest: Task 51 IVF EXPLAIN Counter Cleanup

- head SHA: `fa005394ad5f58341ff8e0f37dec578dfcc9c9b7`
- task bucket: `reviews/task-51`
- packet path: `reviews/task-51/001-ivf-explain-counter-cleanup`
- timestamp: `2026-05-23T04:23:12Z`
- lane: local PG18 smoke before AWS gate
- fixture: synthetic 6-row `ec_ivf` RaBitQ smoke
- storage format: `rabitq`
- rerank mode: `heap_f32`
- surface: isolated local temporary PG18 cluster with `shared_preload_libraries=ecaz`
- index/table layout: isolated one-index-per-table surface

## Artifacts

- `cargo-check-pg18.log`
  - command: `cargo check --lib --no-default-features --features pg18`
  - result: pass, with existing warnings unrelated to this slice.

- `rustfmt-scoped.log`
  - command: `rustfmt --check src/am/common/explain.rs src/am/ec_ivf/scan.rs`
  - result: pass for the files changed by this slice.

- `git-diff-check.log`
  - command: `git diff --check -- src/am/common/explain.rs src/am/ec_ivf/scan.rs`
  - result: pass.

- `run-pg18-ivf-rabitq-explain-smoke.sh`
  - command source for the local backend smoke.
  - starts a temporary PG18 cluster with `shared_preload_libraries=ecaz`,
    creates a RaBitQ IVF index with `rerank=heap_f32`, runs
    `EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)`, and stops the cluster.

- `pg18-ivf-rabitq-explain-smoke.log`
  - command: `bash reviews/task-51/001-ivf-explain-counter-cleanup/artifacts/run-pg18-ivf-rabitq-explain-smoke.sh`
  - result: pass.
  - key result lines:
    - `shared_preload_libraries | ecaz`
    - `Index Scan using task51_ivf_rabitq_smoke_idx`
    - `Centroid Scores: 2`
    - `Selected Lists: 2`
    - `Posting Pages Read: 2`
    - `Postings Visited: 6`
    - `Candidates Emitted: 3`
    - `Rerank Rows: 3`
    - `Heap Blocks Fetched: 1`
    - `Approximate Scan Elapsed Us: 68`
    - `Exact Rerank Elapsed Us: 42`
