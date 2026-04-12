# Review Request: C1 Real-Corpus Latency Capture Hardening

## Context

Branch:
- `task/244-c1-post-vacuum-reporting`

Commit:
- `0ea1801` (`bench: harden real-corpus latency capture`)

Primary scope:
- `scripts/bench_sql_latency.sh`
- `scripts/bench_sql_latency_scratch.sh`
- `docs/RECALL_REAL_CORPUS.md`
- `spec/non-functional/NFR-001-query-latency.md`
- `plan/tasks/10-benchmarks.md`
- `plan/status.md`

This is the first narrow `C1` slice after A5/A6 landed on `origin/main`. The
scratch `pg17` cluster on this host was live, but it did **not** have any
canonical `tqhnsw_real_*` tables loaded, so there was no honest way to claim a
durable post-A6 latency artifact yet. Instead, this checkpoint makes the
existing real-corpus `NFR-001` capture surface trustworthy and self-describing
before the first real recorded run.

Fresh lane-relevant review input came from the local review packet
`review/225-a4-nfr-001-latency-real-corpus/feedback/2026-04-10-01-reviewer.md`.
That review correctly called out that the current script could silently ignore
the requested `ef_search` setting and that its reported `qps` was dominated by
`psql` process spawn overhead rather than server execution time.

## What Landed

### 1. `bench_sql_latency.sh` now applies `ef_search` unambiguously

The real-corpus path used:

```sql
SET LOCAL tqhnsw.ef_search = ...
EXPLAIN (ANALYZE, TIMING, FORMAT JSON) ...
```

inside a fresh `psql` heredoc per query. That left the setting behavior
transaction-shape-dependent. This checkpoint switches the per-query statement to
plain session-level `SET tqhnsw.ef_search = ...`, which is unambiguous for the
rest of that `psql` session and removes the risk of silently benchmarking the
default setting for every cell.

### 2. The reported throughput now reflects server execution, not client spawn cost

The old summary line reported:

```text
qps = queries / wall_seconds
```

for a loop that spawns one `psql` process per query. That number mostly measured
process launch overhead, not PostgreSQL execution throughput.

The script now computes:

- `server_qps = 1000 * n / sum(execution_time_ms)`
- `wall = total client-observed cell wall time`

and emits both in the summary line. This keeps the true server-side rate
visible while preserving the overall cell runtime for operator awareness.

### 3. Real-corpus latency output now carries the artifact metadata it was missing

The real-corpus banner now prints:

- host OS
- CPU model
- RAM
- `shared_buffers`
- `work_mem`
- `max_parallel_workers_per_gather`
- an explicit `--cache-state` label supplied by the operator
- the optional summary-file destination

This makes the stdout artifact self-describing enough for `NFR-001` reporting
without inventing a new harness or a new artifact format.

### 4. The script now guards the current SQL interpolation assumption

The real-corpus query loop still uses the existing
`'${query_line}'::real[]` interpolation shape, but it now refuses to run if the
extracted query literal output contains a single quote. That turns a latent
format assumption into a bench-time failure instead of a silent SQL corruption
path.

### 5. Docs and plan surfaces now match the post-A6 C1 starting point

The adjacent reporting/task surfaces were stale relative to `main`:

- `plan/tasks/10-benchmarks.md` still described SQL benchmark runs as blocked on
  scan / insert / vacuum
- `plan/status.md` still described `C1` as effectively waiting on `A6`

This checkpoint updates those surfaces just enough to reflect reality:

- A3/A5/A6 are merged on `main`
- the first C1 result-capture slice is trustworthy real-corpus `NFR-001`
  latency reporting
- result artifacts are now blocked by staged benchmark corpora and operator
  runs, not by missing runtime features

`docs/RECALL_REAL_CORPUS.md` and `spec/non-functional/NFR-001-query-latency.md`
now also document the new stdout banner plus the `server_qps` interpretation and
show the canonical command shape with `--cache-state` and separate stdout /
summary artifacts.

## Validation

Required checkpoint validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All three are green on this branch.

Additional script sanity:

- `bash -n scripts/bench_sql_latency.sh`
- `bash -n scripts/bench_sql_latency_scratch.sh`
- `scripts/bench_sql_latency.sh --help`

Scratch smoke (tiny ad hoc prefix, not a durable benchmark artifact):

```bash
scripts/bench_sql_latency_scratch.sh \
    --prefix tqhnsw_real_smoke \
    --m 8 \
    --ef-search 4,8 \
    --query-limit 2 \
    --cache-state smoke \
    --output /tmp/tqv_latency_smoke.summary
```

Observed summary lines:

```text
m=8   ef_search=4    n=2     ... server_qps=15.22 wall=0.15s
m=8   ef_search=8    n=2     ... server_qps=15.58 wall=0.15s
```

One note from validation: the first full `cargo pgrx test pg17` run hit an
existing flake in `pg_test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`
(`3` vs `4` reachable nodes on its small fixture). An isolated rerun of that
test passed, and the final full `cargo pgrx test pg17` rerun for this
checkpoint was green. I did not widen this C1 slice to chase that unrelated
flake.

## Review Focus

- Is the real-corpus latency script now semantically trustworthy for the first
  durable `NFR-001` capture, especially around `ef_search` application and
  throughput interpretation?
- Are the new stdout banner and `--cache-state` surface the right minimal way
  to make latency artifacts self-describing without inventing a new harness?
- Are the plan/status/doc updates scoped correctly to this narrow C1 checkpoint,
  or do any of them overstate progress beyond "capture path hardened; real
  corpus still needs to be staged and run"?
