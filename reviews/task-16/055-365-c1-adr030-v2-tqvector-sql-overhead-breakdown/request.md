# Review Request: C1 ADR-030 V2 tqvector SQL Overhead Breakdown

## Context

Packet `364` added a reusable pgvector SQL latency harness and tightened the
planner-level comparison on the isolated grouped `m=16` lane. The result was
unexpectedly harsh:

- pgvector was faster through SQL at `ef=40..128`
- tqvector still looked better in the direct harness

That meant the remaining runtime question was no longer about recall. It was:

> where is tqvector losing time between the direct scan path and the full SQL
> query path?

The likely suspects were:

1. `encode_to_tqvector(...)` query encoding on every SQL call
2. the access-method hot path itself
3. executor / operator / non-AM overhead wrapped around the scan

## Problem

The existing debug profile surface was close, but not precise enough for
`LIMIT 10` SQL comparisons:

- `tests.tqhnsw_debug_scan_hot_path_profile(...)` exposes `amrescan` buckets,
  but not a full end-to-end scan total
- `tests.tqhnsw_debug_scan_profile(...)` measures the full ordered scan, but it
  exhausts all emitted results instead of stopping at the SQL `LIMIT`

So the branch had no direct way to line up:

- SQL `LIMIT 10` latency
- query encoding cost
- internal tqhnsw work to return the first 10 rows

## Planned Slice

Batch the missing seam and the diagnosis harness together:

1. add a limited ordered-scan debug helper that stops after the first `k`
   emitted rows
2. expose that helper in the SQL-visible debug surface and refreshable scratch
   wrappers
3. add a dedicated tqvector overhead-breakdown harness
4. run it on the isolated grouped `m=16` scratch lane over the same `50`
   queries used in packet `364`
5. cross-check the result against `plain-server` SQL timing to rule out
   EXPLAIN-only distortion

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
- `scripts/bench_tqvector_sql_overhead_breakdown.sh`
- `scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh`
- `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`

Concrete changes:

1. refactored the existing ordered-scan debug helper in `scan_debug.rs` so it
   can optionally stop after a caller-provided emitted-row cap
2. kept the existing `debug_profile_ordered_scan(...)` behavior unchanged by
   routing it through the new helper with `None`
3. added `tests.tqhnsw_debug_scan_profile_limited(index_oid, query, limit_count)`
   in the pg_test surface
4. added a pg_test that proves the limited helper stops early and preserves a
   non-exhausted final phase
5. refreshed the scratch debug-helper SQL so the existing scratch cluster can
   call the new limited wrapper without rebuilding the whole fixture schema
6. added a dedicated overhead-breakdown harness that measures, for each
   `ef_search` cell:
   - full verified SQL latency
   - `encode_to_tqvector(...)` latency
   - limited internal scan timing for the first `k` results
   - hot-path `amrescan` subphase timings
   - residual SQL overhead after subtracting the internal scan and encode cost
7. added fake-psql regression coverage for:
   - planner fallback abort-before-timing
   - successful summary generation with encode/profile/hot-path fields

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

Repo checkpoint commands:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Observed result:

- `cargo clippy ...` passed
- `cargo test` failed at the known local PostgreSQL linker layer with the same
  unresolved `pgrx` / Postgres symbols already seen on this workstation:
  `CurrentMemoryContext`, `PG_exception_stack`, `errstart`, `errmsg`, ...
- `cargo pgrx test pg17` failed at the same linker layer with the same symbol
  family

So this checkpoint is green on scripts, compile checks, scratch installation,
and live measurements, but still blocked on the pre-existing local linker issue
for Rust test binaries.

## Measurements

### tqvector grouped `m=16` SQL overhead breakdown

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
  --cache-state warm \
  --warmup-passes 1 \
  --output /tmp/tqvector_grouped_m16only_overhead.summary
```

Observed means:

| ef_search | SQL mean ms | encode mean ms | internal total mean ms | residual SQL over internal ms | residual after encode ms |
|----------:|------------:|---------------:|-----------------------:|------------------------------:|-------------------------:|
| 40  | `5.688` | `0.008` | `1.168` | `4.520` | `4.511` |
| 64  | `7.099` | `0.009` | `1.648` | `5.451` | `5.442` |
| 128 | `9.229` | `0.008` | `2.563` | `6.665` | `6.657` |
| 320 | `14.416` | `0.009` | `4.818` | `9.598` | `9.589` |

Two immediate readouts:

1. `encode_to_tqvector(...)` is effectively noise on this lane
   - about `0.008-0.009ms/query`
2. the missing time is overwhelmingly outside the internal scan path
   - the residual is multi-millisecond at every `ef_search`

### Hot-path subphase detail

The same harness also recorded the grouped hot-path means:

| ef_search | hot amrescan mean ms | graph materialize mean ms | candidate score mean ms |
|----------:|---------------------:|--------------------------:|------------------------:|
| 40  | `1.054` | `0.049` | `0.042` |
| 64  | `1.582` | `0.079` | `0.065` |
| 128 | `2.370` | `0.082` | `0.066` |
| 320 | `4.675` | `0.089` | `0.067` |

The decode / prepare buckets were effectively zero at this scale:

- `query_decode_mean = 0.000ms`
- `prepare_query_mean = 0.000ms`

So the grouped runtime itself is not where the extra `4-10ms` is going.

### `plain-server` cross-check

To rule out `EXPLAIN ANALYZE` as the main cause of the residual, I reran the
same isolated grouped lane through the existing verified launcher in
`plain-server` mode:

| ef_search | plain-server SQL mean ms |
|----------:|-------------------------:|
| 40  | `5.493` |
| 64  | `6.549` |
| 128 | `8.539` |
| 320 | `13.138` |

Compared to the limited internal scan totals above, the plain-server residuals
are still large:

| ef_search | plain-server residual over internal ms |
|----------:|--------------------------------------:|
| 40  | `4.325` |
| 64  | `4.901` |
| 128 | `5.976` |
| 320 | `8.320` |

So the packet `364` gap is **not** mainly EXPLAIN instrumentation overhead.

## Interpretation

This batch changes the diagnosis materially:

- the direct grouped runtime was not lying; the limited internal scan path is
  about `1.2-4.8ms` for the first `10` rows on this lane
- query encoding is negligible
- the big SQL loss sits outside the tqhnsw AM hot path and survives even when
  EXPLAIN instrumentation is removed

That narrows the likely problem surface to the SQL/operator/executor boundary:

1. order-by operator / executor integration cost around the index scan
2. tuple materialization / callback overhead between the AM and SQL
3. planner/executor path shape differences that do not appear in the direct
   harness or the internal debug profile

## Risk / Follow-up

This packet does not improve runtime directly. It tells us where **not** to
spend time:

- not grouped traversal scoring
- not `encode_to_tqvector(...)`
- not the internal `amrescan` hot path

The next useful slice should target the SQL boundary explicitly, for example:

1. instrument or benchmark the operator/executor wrapper cost around the first
   `k` returned rows
2. compare the direct AM path to a narrower executor-facing probe that avoids
   the full SQL order-by surface
3. inspect whether the current operator path is redoing work that the AM has
   already performed for the emitted top-k rows
