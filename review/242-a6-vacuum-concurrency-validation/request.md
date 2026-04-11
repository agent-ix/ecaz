# Review Request: A6 Vacuum Concurrency Validation

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-010-hnsw-vacuum.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/tests.md`
- `review/README.md`

This is the final A6 checkpoint. The vacuum mark/repair/finalize behavior was
already merged; the remaining gap was the explicit concurrent-safety proof from
`FR-010-AC-3` / `FR-022-AC-4` / `TC-215`.

Checkpoint scope:

1. add a SQL-visible `pg_test` helper that exercises the live
   `ambeginscan/amrescan/amgettuple` path and returns a cheap scalar
2. add a repeatable scratch-cluster harness for concurrent INSERT + tqhnsw scan
   + VACUUM over 60 seconds
3. isolate the harness in its own throwaway scratch database so reruns do not
   depend on stale extension state in `postgres`
4. update task/spec/status/review docs to mark A6 complete on `main`

## Scope

- `src/lib.rs`
- `scripts/vacuum_concurrency_scratch.sh`
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `review/README.md`
- `spec/functional/FR-010-hnsw-vacuum.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/tests.md`

## What Landed

### 1. Test-schema SQL hook for real tqhnsw scan execution

The `tests` pg schema now exposes:

- `tests.tqhnsw_debug_scan_result_count(index_oid oid, query real[])`

It validates the relation as a `tqhnsw` index, then calls the existing
`debug_gettuple_scan_heap_tids(...)` helper, which runs the real
`ambeginscan/amrescan/amgettuple` path. The helper returns only a count so the
scratch harness can force live tqhnsw scans cheaply from SQL.

Coverage also now includes:

- `test_tqhnsw_debug_scan_result_count_matches_scan_helper`

That regression proves the SQL wrapper agrees with the existing Rust-side debug
scan helper on the same fixture.

### 2. Dedicated scratch-cluster concurrency harness

`scripts/vacuum_concurrency_scratch.sh` now:

1. connects to the scratch pg17 cluster
2. recreates a dedicated harness database
3. creates `tqvector` fresh in that database
4. seeds a tqhnsw fixture table and index
5. runs one INSERT worker, one VACUUM/delete worker, and two tqhnsw scan
   workers concurrently for a configurable duration
6. fails fast on worker errors and prints worker logs on failure

The harness uses its own throwaway database instead of the shared `postgres`
database, which avoids stale-extension-state false failures after reinstalling a
`pg_test` build.

### 3. A6 docs now close the runtime lane

The planning/spec/test/review surfaces now record that:

- A6 is complete on `main`
- `TC-215` is currently satisfied by the scratch harness
- the concurrency proof is intentionally a multi-session harness rather than a
  normal `#[pg_test]`
- the next runtime lane can move past vacuum correctness

## Validation

Standard checkpoint validation:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Concurrency validation:

- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx start pg17`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`
- `scripts/vacuum_concurrency_scratch.sh --duration 5`
- `scripts/vacuum_concurrency_scratch.sh --duration 60`

Observed 60-second harness result:

- `vacuum concurrency harness passed`
- `duration_seconds=60`
- `final_live_rows=10844`
- `final_scan_count=10844`

## Review Focus

- Is the dedicated scratch-database harness the right durability boundary for
  `TC-215`, or should this proof move elsewhere later?
- Is the `pg_test`-only SQL count wrapper an acceptable narrow seam for driving
  real tqhnsw scans from the harness?
- Does any A6 concurrency risk remain materially unexercised by the current
  INSERT + scan + VACUUM worker mix?
