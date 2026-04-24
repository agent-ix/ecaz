# 588-c1 PG18 Parallel Pathlist Diagnostics

## Request

Review the PG18 planner pathlist diagnostic checkpoint at head
`70f662a83dff5551ef41290b2f24baba3f9afdc7`.

This checkpoint adds a PG18-only `set_rel_pathlist_hook` diagnostic surface.
The hook records the latest base relation that owns an `ec_hnsw` index and
exposes a backend-local snapshot through:

- `ec_hnsw_reset_planner_path_snapshot()`
- `ec_hnsw_planner_path_snapshot()`

`ecaz dev test pg18-parallel-scan --diagnose-planner` now resets the snapshot,
runs the parallel-candidate ordered `EXPLAIN`, and prints the pathlist snapshot
before later control plans can overwrite it.

## Result

On the normal PG18 branch build, with `amcanparallel` still disabled, the
pathlist snapshot distinguishes the current blocker:

- the fixture relation is parallel-eligible:
  `consider_parallel=true rel_parallel_workers=4`
- the relation has one `ec_hnsw` index, but the index is not advertised as
  parallel-capable yet: `ec_hnsw_index_count=1 amcanparallel_seen=false`
- PostgreSQL generated one partial path for the relation:
  `partial_path_count=1`
- PostgreSQL generated no partial `ec_hnsw` index path:
  `partial_ec_hnsw_index_path_count=0`
- the serial ordered `ec_hnsw` path has pathkeys:
  `best_plain_ec_hnsw ... parallel_workers=0 pathkeys=1`

So the current normal-build result is not "partial `IndexPath` generated and
then dominated"; it is "partial `ec_hnsw` index path absent because
`amcanparallel` is still false." The cost snapshot remains useful for the next
activation slice because it shows the normal fixture is startup-cost dominated
once a local `amcanparallel` probe is attempted.

## Artifacts

- `artifacts/pg18-parallel-pathlist-diagnostics.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo check`
- `cargo test -p ecaz-cli`
- `cargo test choose_best_by_total_keeps_lowest_total_cost`
- `cargo test test_pg18_planner_path_snapshot_records_ec_hnsw_paths -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/588-c1-pg18-parallel-pathlist-diagnostics/artifacts/pg18-parallel-pathlist-diagnostics.log`

The full `cargo test` run used the repo default PG18 feature set and reported
`PgConfig("pg18")` for the pgrx-backed section.

## Review Focus

- Is the hook safe and narrow enough to keep as a PG18 diagnostic surface?
- Are the pathlist counters sufficient for the next local `amcanparallel=true`
  probe?
- Should the next slice run that probe and use this same snapshot to determine
  whether the resulting partial `ec_hnsw` path has workers and pathkeys or is
  dominated by cost?
