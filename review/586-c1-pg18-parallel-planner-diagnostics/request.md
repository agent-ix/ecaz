# 586-c1 PG18 Parallel Planner Diagnostics

## Request

Review the PG18 planner-diagnostic checkpoint at head
`565aba898a01c7b349a4f5fcc975141c76ecb263`.

This checkpoint adds `ecaz dev test pg18-parallel-scan --diagnose-planner`,
which prints planner settings, relation/operator/opclass catalog facts, the
existing `ec_hnsw_planner_integration_snapshot`, and `EXPLAIN (VERBOSE,
FORMAT JSON)` plans for:

- serial ordered index candidate with `max_parallel_workers_per_gather=0`
- parallel-candidate ordered index query
- parallel seqscan worker-control query
- forced ordered seqscan control query

It also moves `ecaz dev test pgrx` to default to PG18. PG17 remains available
explicitly with `--pg 17` as a compatibility lane.

The same command now accepts `--log-output <path>` so packet artifacts can be
captured through the CLI instead of shell redirection.

## Result

On the normal branch build, with `amcanparallel` still disabled, the diagnostic
fixture proves PostgreSQL can parallelize the relation and the same `<#>`
ORDER BY expression outside the index AM path:

- the worker-control seqscan launches 4 workers
- the forced ordered seqscan is `Limit -> Gather Merge -> Sort -> Seq Scan`,
  with the child seqscan marked `Parallel Aware: true`
- the ordered index candidate remains a serial `Index Scan`, with
  `Parallel Aware: false`
- serial and candidate ordered IDs match exactly
- `<#>(ecvector, real[])` resolves to a parallel-safe immutable function

The current blocker therefore appears to be AM partial-index-path generation,
costing, or path selection. The diagnostic output argues against relation
parallel eligibility or ORDER BY expression parallel safety being the blocker.

I also ran a temporary local `amcanparallel = cfg!(feature = "pg18")` probe
while diagnosing this, then reverted it and reinstalled the normal PG18 build.
That probe is not part of this checkpoint.

## Artifacts

- `artifacts/pg18-parallel-planner-diagnostics.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -p ecaz-cli -- dev test pgrx --help`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/586-c1-pg18-parallel-planner-diagnostics/artifacts/pg18-parallel-planner-diagnostics.log`

The full `cargo test` run used the repo default PG18 feature set and reported
`PgConfig("pg18")` for the pgrx-backed section.

## Review Focus

- Is the `--diagnose-planner` output sufficient for the next planner-visible
  activation step?
- Are the JSON plans and catalog probes the right facts to keep in the CLI
  rather than in ad hoc SQL notes?
- Is defaulting the CLI-owned pgrx test surface to PG18 the right local
  expression of PG17-as-compatibility-test?
