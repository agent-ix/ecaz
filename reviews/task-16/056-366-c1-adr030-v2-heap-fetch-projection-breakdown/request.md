# Review Request: C1 ADR-030 V2 Heap Fetch / Projection Breakdown

## Context

Packet `365` narrowed the tqvector SQL latency gap on the isolated grouped
`m=16` lane:

- `encode_to_tqvector(...)` was effectively free
- the internal `LIMIT 10` AM scan was much cheaper than the full SQL query
- the missing `3.5-9.6ms` sat outside the direct tqhnsw scan path

That still left an important uncertainty:

> is the remaining gap mostly heap fetch / tuple-slot work after `amgettuple`,
> or is it still higher up in executor / SQL integration?

## Problem

The branch had AM-only and SQL-level timing, but nothing in between.

Specifically, packet `365` could not answer:

1. how much time Postgres spends fetching heap tuples for the first `10` index
   results
2. whether simple projection on the fetched slot explains the remaining gap
3. whether the SQL residual survives after measuring an executor-like
   slot-fetch path instead of just the bare AM path

## Planned Slice

Batch the missing seam and the updated measurement together:

1. add a pg_test-visible debug helper that runs a generic index scan,
   fetches the first `k` tuples into a heap slot, and optionally projects a
   requested attribute
2. refresh the scratch helper SQL so the live scratch cluster can call it
3. extend the tqvector SQL-overhead harness with an executor-like slot-fetch
   pass
4. rerun the isolated grouped `m=16` lane over the same `50` real queries
5. compare the new executor-like total against the existing internal AM total
   and the full SQL timing

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
- `scripts/bench_tqvector_sql_overhead_breakdown.sh`
- `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`

Concrete changes:

1. added `debug_profile_ordered_scan_with_heap_fetch(...)` in `scan_debug.rs`
2. implemented that helper with the generic Postgres scan path:
   - `index_beginscan(...)`
   - `index_rescan(...)`
   - `index_getnext_slot(...)`
   - optional `slot_getattr(...)`
3. exposed the helper as
   `tests.tqhnsw_debug_scan_heap_fetch_profile(index_oid, query, limit_count, project_attnum)`
4. added a pg_test that proves the helper stops at the requested limit and
   projects the requested heap attribute on a small fixture
5. refreshed the scratch wrapper SQL so the existing scratch cluster can call
   the new helper without rebuilding the fixture schema
6. extended the tqvector SQL-overhead harness to record, for each `ef_search`
   cell:
   - full verified SQL latency
   - internal limited AM scan latency
   - executor-like slot-fetch total
   - slot-fetch-only time inside `index_getnext_slot(...)`
   - projection time from `slot_getattr(...)`
   - residual SQL over the executor-like total
7. added fake-psql regression coverage for the new helper requirement and the
   new summary fields

Important note:

- the first local attempt manually mixed `tqhnsw_amgettuple(...)` with
  `index_fetch_heap(...)` and crashed the backend on the scratch lane
- this packet fixes that by routing through the generic scan API that the
  executor already uses (`index_rescan` + `index_getnext_slot`)

## Validation

Script validation:

- `bash -n scripts/bench_tqvector_sql_overhead_breakdown.sh scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh`
- `python3 -m unittest scripts.tests.test_bench_tqvector_sql_overhead_breakdown`

Observed result:

- both passed

Compile validation:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`

Observed result:

- both passed

Scratch install / wrapper refresh:

- `./scripts/install_adr030_pg17_pg_test.sh`
- `./scripts/refresh_adr030_scratch_debug_helpers.sh`

Observed result:

- both succeeded on the scratch cluster

Live helper smoke:

- `./scripts/pg17_scratch_psql.sh --sql "SET tqhnsw.ef_search = 40; SELECT * FROM tests.tqhnsw_debug_scan_heap_fetch_profile(...);"`

Observed result:

- succeeded on the real grouped `m=16` scratch index after the generic scan
  fix

Repo checkpoint commands:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Observed result:

- `cargo clippy ...` passed
- `cargo test` failed at the same pre-existing local PostgreSQL linker layer
  with unresolved `pgrx` / Postgres symbols such as `CurrentMemoryContext`,
  `PG_exception_stack`, `errstart`, `errmsg`, ...
- `cargo pgrx test pg17` failed at the same linker layer with the same symbol
  family

So this checkpoint is green on scripts, compile checks, scratch installation,
live scratch measurements, and clippy, but still blocked on the workstation’s
existing Rust test-binary linker issue.

## Measurements

### tqvector grouped `m=16` heap-fetch / projection breakdown

Command:

```bash
bash scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh \
  --corpus-table scratch_tqhnsw_real_50k_grouped_m16only_corpus \
  --query-table tqhnsw_real_50k_queries_50 \
  --index-name scratch_tqhnsw_real_50k_grouped_m16only_idx \
  --bits 4 \
  --seed 42 \
  --ef-search 40,64,128,320 \
  --query-limit 50 \
  --result-limit 10 \
  --project-attnum 1 \
  --cache-state warm \
  --warmup-passes 1 \
  --output /tmp/tqvector_grouped_m16only_overhead_heap.summary
```

Observed means:

| ef_search | SQL mean ms | internal total mean ms | executor-like total mean ms | slot-fetch mean ms | projection mean ms | residual SQL over executor-like ms |
|----------:|------------:|-----------------------:|----------------------------:|-------------------:|-------------------:|-----------------------------------:|
| 40  | `4.696` | `1.141` | `0.964` | `0.009` | `0.000` | `3.733` |
| 64  | `5.560` | `1.257` | `1.346` | `0.008` | `0.000` | `4.214` |
| 128 | `7.188` | `2.006` | `2.062` | `0.033` | `0.000` | `5.126` |
| 320 | `10.811` | `4.105` | `4.030` | `0.031` | `0.000` | `6.781` |

Two immediate readouts:

1. executor-like slot fetch tracks the internal AM scan very closely
   - the difference is about `-0.177 / +0.089 / +0.056 / -0.076 ms`
   - that is effectively noise at this scale
2. simple projection on the fetched `id` slot is negligible
   - `projection_mean = 0.000ms` at every measured cell

### What this rules out

This packet materially narrows the SQL gap diagnosis again.

It now rules out:

1. query encoding
2. grouped traversal scoring / direct AM hot-path work
3. heap-slot fetch of the first `10` rows
4. simple slot projection of the fetched `id` column

The missing time still sits above that executor-like slot-fetch path.

## Interpretation

The direct conclusion is:

> the current tqvector SQL gap is not explained by heap fetch or simple tuple
> projection after `amgettuple`

The measured executor-like total is essentially the same as the limited AM
scan total, but the full SQL query is still `3.7-6.8ms` slower on this lane.

That strongly suggests the remaining cost is higher in the stack:

1. executor node / callback integration around the ordered index scan
2. SQL-level tuple materialization / result shaping not reproduced by the
   direct slot-fetch helper
3. possibly EXPLAIN / planner path overhead on top of the same underlying scan
   shape

Packet `365`’s earlier plain-server cross-check is still relevant here:
the residual also survived when EXPLAIN instrumentation was reduced, so this
does not look like a pure EXPLAIN artifact either.

## Risk / Follow-up

This packet does not improve end-user latency directly. It reduces the search
space for the next runtime work:

- do not spend more time on grouped traversal scoring for this SQL gap
- do not spend more time on heap fetch / simple slot projection for this SQL
  gap
- next useful investigation should move to the SQL / executor boundary above
  the index scan, not back into the AM core
