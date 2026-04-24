# 587-c1 PG18 Parallel Cost Diagnostics

## Request

Review the PG18 planner cost diagnostic checkpoint at head
`3ee1a522a366269a1156dd0a1f21a284f7eb369e`.

This checkpoint extends `ecaz dev test pg18-parallel-scan --diagnose-planner`
with an `ec_hnsw cost snapshot` section sourced from the existing
`ec_hnsw_index_cost_snapshot()` SQL function. The section records the fixture's
effective `ef_search`, graph metadata, planner cost constants, and modeled
startup/run cost split.

The command continues to use `--log-output <path>` for packet artifacts, so the
raw output does not rely on shell redirection.

## Result

On the normal PG18 branch build, with `amcanparallel` still disabled, the
diagnostic now shows the ordered `ec_hnsw` plan is dominated by startup-heavy AM
costing:

- `modeled_startup_cost=4015.515`
- `modeled_total_cost=4015.515`
- `modeled_run_cost=0.000`
- `startup_fraction=1.000000`

The same run still confirms that PostgreSQL can launch workers for the fixture
and can parallelize the ordered expression through the forced seqscan control:

- worker-control seqscan launched 4 workers
- forced ordered seqscan used `Gather Merge` over a parallel-aware seqscan
- ordered `ec_hnsw` candidate remained a serial `Index Scan`
- serial and candidate ordered IDs matched exactly

This strengthens the next activation hypothesis: before `amcanparallel` can
stay enabled, the partial index path needs either direct pathlist confirmation
or a cost model change that gives PostgreSQL a real modeled advantage for the
parallel index path.

## Artifacts

- `artifacts/pg18-parallel-cost-diagnostics.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/587-c1-pg18-parallel-cost-diagnostics/artifacts/pg18-parallel-cost-diagnostics.log`

The full `cargo test` run used the repo default PG18 feature set and reported
`PgConfig("pg18")` for the pgrx-backed section.

## Review Focus

- Are these modeled startup/run fields enough to guide the next planner
  activation slice?
- Should the next checkpoint add a PG18 pathlist hook to distinguish "partial
  `IndexPath` absent" from "partial `IndexPath` generated but dominated"?
- Is the derived `startup_fraction` line useful in the CLI artifact, or should
  the diagnostic stay strictly to raw extension snapshot fields?
