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

This packet prepares the required measurement run; it does not yet claim the
full Phase 3 benchmark results.

## Validation

Packet-local artifacts are under
`reviews/task-71/003-worker-curve/artifacts/`.

- `cargo test -p ecaz-cli commands::bench::suite::tests::`
  - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 364 filtered out`
- `cargo test -p ecaz-cli commands::corpus::load::tests::table_reloption_set_clause_strips_create_table_prefix`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 399 filtered out`
- `cargo run -p ecaz-cli -- bench suite run --config reviews/task-71/003-worker-curve/suite.json --dry-run --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json`
  - Dry run wrote the manifest and rendered all steps.
  - Load steps include `PGOPTIONS="-c max_parallel_maintenance_workers=N"`.
  - Load steps include `--table-reloption parallel_workers=N`.
  - Explicit load/recall/storage paths render under
    `reviews/task-71/003-worker-curve/artifacts/`, not literal
    `${artifact_dir}`.
- `cargo run -p ecaz-cli -- dev sql --pg 18 --db tqvector_bench --socket-dir /Users/peter/.pgrx ...`
  - Preflight confirmed `tqvector_bench` has extension `ecaz 0.1.1`
    and access method `ec_ivf`.

Failed full-suite attempts are retained as packet-local artifacts because they
explain the runner fixes:

- `suite-run.log`: non-escalated socket access failed with
  `Operation not permitted`.
- `suite-run-escalated.log`: child load commands did not inherit an explicit
  host/hostaddr.
- `suite-run-escalated-hosted.log`: the prior suite shape passed
  `parallel_workers` as an IVF index reloption; PostgreSQL rejected it with
  `ERROR: unrecognized parameter "parallel_workers"`.

## Review Focus

- Whether `pgoptions` on load steps is narrow enough for the measurement need.
- Whether `table_reloptions` is the right suite/load surface for heap table
  `parallel_workers=N`.
- Whether `${artifact_dir}` templating should cover any additional explicit
  path fields before the full run.
- Whether the suite matrix matches Task 71 Phase 3: worker counts 1/2/4/8,
  real10k/25k/50k/100k, fixed recall@10 point, storage checks, and worker
  counter evidence.
