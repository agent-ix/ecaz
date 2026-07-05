# Task 71 Review Request: Worker-Curve Suite Setup

## Scope

This packet sets up the Phase 3 measurement lane for IVF parallel build. The
code commits under review are:

- `0bb998345 Support PGOPTIONS in suite load steps`
- `dcd45b2d8 Support suite table reloptions for corpus loads`

The slice:

- Adds `pgoptions` support to `ecaz bench suite` load steps so load/build
  commands can set `max_parallel_maintenance_workers=N` through libpq startup
  GUCs.
- Adds `table_reloptions` support to suite load steps and `ecaz corpus load`
  so PostgreSQL table storage options such as `parallel_workers=N` can be
  applied to the corpus heap table rather than passed as IVF index options.
- Extends `${artifact_dir}` templating to explicit suite path fields used by
  load, recall, storage, explain, latency, compare, sidecar, and raw steps.
- Adds `reviews/task-71/003-worker-curve/suite.json`, a dry-run-validated
  worker matrix for real10k/25k/50k/100k at requested workers 1/2/4/8.
- Includes raw before/after worker-counter steps using
  `pg_stat_get_db_parallel_workers_launched`.

Follow-up after reviewer feedback:

- Adds `20d4db545 Wire IVF parallel build scan callbacks`, which wires
  `ec_ivf` to the common `amestimateparallelscan`, `aminitparallelscan`,
  and `amparallelrescan` callbacks while keeping `amcanparallel = false`.
  This fixes the zero-worker symptom found in the pre-fix suite run.
- Adds `99f7a2edc Add IVF parallel build probe command`, which moves the
  real10k IVF parallel-build setup/probe behind the reusable
  `ecaz dev test ivf-parallel-build-probe` surface and emits the last build's
  IVF timing row through `ecaz corpus load` / `ecaz bench suite`.
- Tightens `test_ec_ivf_parallel_build_workers_and_counts` to set both
  `max_parallel_maintenance_workers` and `max_parallel_workers`, matching
  the HNSW parallel-build test setup.
- Updates `suite.json` so every load step sets both
  `max_parallel_maintenance_workers=N` and `max_parallel_workers=N`.
- Extends `ecaz bench suite` load steps with
  `capture_parallel_workers: true`, which samples
  `pg_stat_get_db_parallel_workers_launched` immediately before and after a
  load step, stores before/after/delta in `suite-manifest.json`, and emits a
  `parallel_workers` row in `results.jsonl`.
- Adds `faa22c2c3 Tighten IVF build test workflow`, which moves Task 71 matrix
  cleanup into `ecaz dev test ivf-parallel-build-clean` and makes the
  pq_fastscan build model carry its SRHT signs instead of reacquiring them
  from the quantizer cache during posting encoding.

This packet now records the fresh post-fix worker-curve run. It shows IVF
parallel build workers are launching and heap ingest scales, but full
`CREATE INDEX` wall time is still not a multi-x win.

## Validation

Packet-local artifacts are under
`reviews/task-71/003-worker-curve/artifacts/`.

- `cargo test -p ecaz-cli commands::bench::suite::tests::`
  - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 364 filtered out`
- `cargo test -p ecaz-cli commands::corpus::load::tests::table_reloption_set_clause_strips_create_table_prefix`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 399 filtered out`
- `cargo run -p ecaz-cli -- bench suite run --config reviews/task-71/003-worker-curve/suite.json --dry-run --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json`
  - Dry run wrote the manifest and rendered all steps.
  - Load steps include
    `PGOPTIONS="-c max_parallel_maintenance_workers=N -c max_parallel_workers=N"`.
  - Load steps include `--table-reloption parallel_workers=N`.
  - Explicit load/recall/storage paths render under
    `reviews/task-71/003-worker-curve/artifacts/`, not literal
    `${artifact_dir}`.
- `cargo run -p ecaz-cli -- dev sql --pg 18 --db tqvector_bench --socket-dir /Users/peter/.pgrx ...`
  - Preflight confirmed `tqvector_bench` has extension `ecaz 0.1.1`
    and access method `ec_ivf`.
- `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts`
  - Non-escalated run failed during pgrx extension install with
    `Operation not permitted` writing into the PostgreSQL extension
    directory; the test body did not run.
  - Escalated run passed:
    `test tests::pg_test_ec_ivf_parallel_build_workers_and_counts ... ok`.
  - Because the test asserts `requested_workers == 2` and
    `workers_launched >= 1`, this validates that the callback wiring can
    launch IVF parallel-build workers in the PG18 pg_test environment.
- `cargo test -p ecaz-cli commands::bench::suite::tests::`
  - `test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 364 filtered out`
  - Covers `capture_parallel_workers` config parsing, worker-counter output
    parsing, and `parallel_workers` result-row emission.
- `cargo check --no-default-features --features pg18`
  - passed.
- `./target/debug/ecaz dev test ivf-parallel-build-probe --host /Users/peter/.pgrx --port 28818 --drop-first`
  - passed without approval escalation through the CLI-owned DB test setup.
  - Artifact:
    `artifacts/probe-load-real10k-w2-after-loader-timing.log`.
  - Key line:
    `requested_workers=2 workers_launched=2 heap_tuples=10000 index_tuples=10000`.
  - This verifies the installed PG18 IVF build path is launching parallel
    build workers after the stale-dylib root cause was corrected; it is not
    parallel scan evidence.
- `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/003-worker-curve/artifacts/task71-clean-before-final-suite.log dev test ivf-parallel-build-clean --include-probe`
  - passed without approval escalation through the CLI-owned DB setup surface.
  - Artifact:
    `artifacts/task71-clean-before-final-suite.log`.
  - Key line:
    `[ivf-clean] dropped 17 prefixes`.
- `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/003-worker-curve/artifacts/suite-run-final.log bench suite run --config reviews/task-71/003-worker-curve/suite.json`
  - passed without approval escalation.
  - Artifacts:
    `artifacts/suite-run-final.log`, `artifacts/suite-manifest.json`,
    `artifacts/results.jsonl`, plus per-cell load/recall/storage logs.
  - Worker launch rows from `ec_ivf_build_timing`:
    real10k/25k/50k/100k all recorded `1/1`, `2/2`, `4/4`, and `8/7`
    requested/launched workers for worker counts 1/2/4/8.
  - Full build-index seconds:
    - real10k: `0.464140`, `0.436080`, `0.414400`, `0.411170`
    - real25k: `0.721680`, `0.652020`, `0.621400`, `0.612060`
    - real50k: `1.160000`, `1.020000`, `0.937100`, `0.922410`
    - real100k: `2.630000`, `2.220000`, `2.070000`, `2.030000`
  - Best full-build speedup over w1 is therefore about 1.13x, 1.18x,
    1.26x, and 1.30x respectively; this does not meet Task 71's multi-x
    exit criterion.
  - Heap ingest does scale inside the parallel build path. For real100k,
    `heap_ingest_us` moves from `877228` at w1 to `274479` at w8
    (~3.19x), while leader-side train/stage/flush work keeps full
    `CREATE INDEX` from scaling comparably.
  - Recall@10 matches the Task 31 baselines in every worker cell:
    real10k `1.0000`, real25k `0.9990`, real50k `1.0000`, real100k
    `0.9820`.
  - ec_ivf index `size_bytes` are invariant across workers:
    real10k `2726298`, real25k `5557453`, real50k `10171187`,
    real100k `20342374`.

The pre-fix hosted full-suite run completed, but it is not Phase 3 evidence
for the fixed implementation:

- `parallel-workers-before.log` and `parallel-workers-after.log` both report
  `tqvector_bench  0`.
- The run therefore measured serial/fallback behavior despite requested
  worker counts.
- The post-fix suite has now been rerun. The old database-level
  `pg_stat_get_db_parallel_workers_launched` counter still reports zero in
  this environment, so the accepted per-build worker evidence is the
  `ec_ivf_build_timing` row emitted from each load log.

Failed full-suite attempts are retained as packet-local artifacts because they
explain the runner fixes:

- `suite-run.log`: non-escalated socket access failed with
  `Operation not permitted`.
- `suite-run-escalated.log`: child load commands did not inherit an explicit
  host/hostaddr.
- `suite-run-escalated-hosted.log`: the prior suite shape passed
  `parallel_workers` as an IVF index reloption; PostgreSQL rejected it with
  `ERROR: unrecognized parameter "parallel_workers"`.

## Task 31 Baselines and Manifest Prefixes

The suite reuses the staged Task 31 DBPedia M5 corpus/query files and keeps the
same fixed recall points for each scale:

| scale | Task 31 packet | reloptions | recall@10 |
|---|---|---|---:|
| real10k | `reviews/task-31/005-30169-task31-m5-pqg8-10k-load-baseline/` | `nlists=64,nprobe=48,rerank_width=750` | `1.0000` |
| real25k | `reviews/task-31/006-30170-task31-m5-pqg8-25k-load-baseline/` | `nlists=64,nprobe=48,rerank_width=750` | `0.9990` |
| real50k | `reviews/task-31/007-30171-task31-m5-pqg8-50k-load-baseline/` | `nlists=64,nprobe=48,rerank_width=750` | `1.0000` |
| real100k | `reviews/task-31/009-30173-task31-m5-pqg8-100k-n128-w500-baseline/` | `nlists=128,nprobe=48,rerank_width=500` | `0.9820` |

The adjacent Task 31 100k fixed-scale packet
`reviews/task-31/008-30172-task31-m5-pqg8-100k-load-baseline/` records
`nlists=64,nprobe=48,rerank_width=750` at recall@10 `0.9940`; packet 009 is
the directly comparable 100k setting used by this Task 71 matrix.

`allow_manifest_mismatch: true` is intentional for these load steps. The input
paths still point at the Task 31 staged corpus/query TSVs and manifests under
`data/task31_m5_dbpedia_staged/`, but the suite uses isolated Task 71 table
prefixes such as `task71_real25k_w4`. The manifest verifier therefore warns
only because the manifest prefix is `ec_hnsw_real_*` while the destination
table prefix is Task 71-specific. The load logs retain the warning and the
inspected source paths; no corpus/query content substitution is being used.

## Review Focus

- Whether `pgoptions` on load steps is narrow enough for the measurement need.
- Whether `table_reloptions` is the right suite/load surface for heap table
  `parallel_workers=N`.
- Whether `${artifact_dir}` templating should cover any additional explicit
  path fields before the full run.
- Whether the suite matrix matches Task 71 Phase 3: worker counts 1/2/4/8,
  real10k/25k/50k/100k, fixed recall@10 point, storage checks, and worker
  counter evidence.
- Whether the post-callback suite rerun's `parallel_workers` result rows are
  sufficient per-build worker-counter evidence.
