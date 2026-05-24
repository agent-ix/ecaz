# Artifact Manifest: IVF Bound-Prune Limit Guard

- head SHA: `aea43e0adf8205a0689af465941e89d1d68780f2`
- task bucket: `reviews/task-51/004-ivf-bound-prune-limit-guard`
- timestamp: `2026-05-23T05:05:30Z`
- lane: `ec_ivf` RaBitQ local smoke
- storage format: `rabitq`
- rerank modes: `off`, `heap_f32`
- fixture: synthetic 256-row local PG18 table
- isolation: one index per table surface, with the no-rerank index dropped
  before creating the heap-f32 index
- AWS: not used
- vchord: not used
- pgvectorscale: not used

## Artifacts

- `cargo-check-pg18.log`
  - command: `cargo check --lib --no-default-features --features pg18`
  - result: passed
  - note: existing unrelated warnings remain in `src/am/mod.rs` and
    `src/am/ec_ivf/build.rs`.

- `cargo-test-pre-rerank-limit-no-run.log`
  - command: `cargo test --lib pre_rerank_candidate_limit_requires_heap_f32_positive_width --no-run --no-default-features --features pg18`
  - result: passed
  - note: compile-only validation is intentional for this focused unit target;
    local PG18 smoke covers runtime extension behavior.

- `rustfmt-scan.log`
  - command: `rustfmt --check src/am/ec_ivf/scan.rs`
  - result: passed
  - note: rustfmt emitted existing stable-channel warnings for unstable config
    keys.

- `diff-check-scan.log`
  - command: `git diff --check -- src/am/ec_ivf/scan.rs`
  - result: passed

- `cargo-pgrx-install-pg18.log`
  - command: `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
  - result: passed

- `run-pg18-ivf-rabitq-bound-prune-smoke.sh`
  - command source for the local PG18 smoke
  - creates a 256-row table and an `ec_ivf` RaBitQ no-rerank index
  - runs `EXPLAIN (ecaz, ANALYZE)` with `LIMIT 220`
  - drops the no-rerank index, creates a heap_f32 RaBitQ index, and runs
    `EXPLAIN (ecaz, ANALYZE)` with `LIMIT 3`

- `pg18-ivf-rabitq-bound-prune-smoke.log`
  - command: `bash reviews/task-51/004-ivf-bound-prune-limit-guard/artifacts/run-pg18-ivf-rabitq-bound-prune-smoke.sh`
  - result: passed
  - no-rerank key lines:
    - `Postings Pruned By Bound: 0`
    - `Candidates Emitted: 220`
    - `Rerank Rows: 0`
    - `no_rerank_limit_220_count = 220`
  - heap_f32 key lines:
    - `Postings Pruned By Bound: 252`
    - `Candidates Emitted: 3`
    - `Rerank Rows: 3`
    - `Heap Blocks Fetched: 1`
    - `heap_f32_limit_3_count = 3`
