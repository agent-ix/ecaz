# Review Request: C1 ADR-030 V2 pgvector SQL Latency Harness

## Context

Packet `363` established two things on the scratch `50k` real-corpus lane:

- pgvector HNSW is about `6x` larger on disk than tqvector on the same data
- the direct runtime comparison is a real trade-off:
  - tqvector grouped-v2 `m=16` is faster
  - pgvector `m=16` is much more accurate

That packet also left an explicit follow-up:

> build a proper pgvector latency harness that mirrors the tqvector verified SQL
> launcher more closely

The ad hoc pgvector probe SQL was enough to answer the first question, but not
good enough for a repeatable planner-level baseline.

## Problem

The repo had no reusable SQL latency harness for pgvector HNSW.

That left the branch with an asymmetry:

- tqvector had a guarded, planner-verified SQL launcher
- pgvector only had one-off scratch SQL probes

Without a reusable pgvector launcher, the branch could not make a cleaner
apples-to-apples end-to-end SQL comparison on the same corpus and query subset.

## Planned Slice

Batch the harness and the first measured readout together:

1. add a pgvector SQL latency launcher that mirrors the repo’s real-corpus
   tqvector SQL harness shape
2. add a scratch wrapper matching the existing scratch launcher pattern
3. add regression coverage around planner verification and per-cell execution
4. run the new harness on the live `pgvector_real_50k_m16_idx` scratch lane
5. rerun the isolated grouped tqvector `m=16` verified SQL lane on the same
   cluster state
6. compare the resulting SQL latency surfaces directly

## Implementation

Added:

- `scripts/bench_pgvector_sql_latency.sh`
- `scripts/bench_pgvector_sql_latency_scratch.sh`
- `scripts/tests/test_bench_pgvector_sql_latency.py`

Concrete behavior:

1. `bench_pgvector_sql_latency.sh` is a real-corpus-only SQL launcher for
   pgvector HNSW:
   - requires `--corpus-table`, `--query-table`, and `--index-name`
   - uses `SET hnsw.ef_search = ...`
   - verifies each measured cell still plans onto the expected pgvector index
   - supports `per-query` and `per-cell` backend reuse modes
   - supports `EXPLAIN (ANALYZE, FORMAT JSON)` timing and a plain server-side
     timing mode
2. `bench_pgvector_sql_latency_scratch.sh` mirrors the repo-local scratch
   wrapper pattern and binds the same pg17 scratch socket defaults used by the
   existing tqvector launchers
3. the new Python regression tests cover:
   - abort-before-timing on planner fallback for a single `ef_search` cell
   - successful per-cell execution with warmup plus timing output across
     multiple `ef_search` values

## Validation

Harness checks:

- `bash -n scripts/bench_pgvector_sql_latency.sh scripts/bench_pgvector_sql_latency_scratch.sh`
- `python3 -m unittest scripts.tests.test_bench_pgvector_sql_latency`

Observed result:

- both passed

Repo checkpoint commands:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Observed result:

- `cargo clippy ...` passed
- `cargo test` failed at the existing local PostgreSQL linker layer with the
  same unresolved `pgrx`/Postgres symbols already seen on this workstation:
  `CurrentMemoryContext`, `PG_exception_stack`, `errstart`, `errmsg`, ...
- `cargo pgrx test pg17` failed at the same linker layer with the same symbol
  family

So this checkpoint is green on script validation and measurements, but still
blocked on the known local linker issue for the Rust test binaries.

## Measurements

### New pgvector SQL harness readout

Command:

```bash
bash scripts/bench_pgvector_sql_latency_scratch.sh \
  --corpus-table pgvector_real_50k_corpus \
  --query-table tqhnsw_real_50k_queries_50 \
  --index-name pgvector_real_50k_m16_idx \
  --dim 1536 \
  --ef-search 40,64,128,320 \
  --query-limit 50 \
  --cache-state warm \
  --warmup-passes 1 \
  --output /tmp/pgvector_real_50k_m16_sql.summary
```

Observed mean SQL latencies:

| ef_search | pgvector `m=16` mean SQL latency ms |
|----------:|------------------------------------:|
| 40  | `4.080` |
| 64  | `4.309` |
| 128 | `7.465` |
| 320 | `13.389` |

### Current isolated grouped tqvector `m=16` verified SQL rerun

Command:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix scratch_tqhnsw_real_50k_grouped_m16only \
  --corpus-table scratch_tqhnsw_real_50k_grouped_m16only_corpus \
  --query-table tqhnsw_real_50k_queries_50 \
  --index-name scratch_tqhnsw_real_50k_grouped_m16only_idx \
  --m 16 \
  --ef-search 40,64,128,320 \
  --query-limit 50 \
  --cache-state warm \
  --warmup-passes 1 \
  --output /tmp/tqvector_grouped_m16only_sql_rerun.summary
```

Observed mean SQL latencies:

| ef_search | tqvector grouped `m=16` mean SQL latency ms |
|----------:|--------------------------------------------:|
| 40  | `5.625` |
| 64  | `6.607` |
| 128 | `8.523` |
| 320 | `13.466` |

### Interpretation

The important read is that the planner-level comparison is not the same as the
direct harness comparison from packet `363`.

At the SQL layer on this isolated `m=16` lane:

- pgvector is faster at `ef=40`, `64`, and `128`
- the two lanes are effectively tied by `ef=320`

Representative comparison:

- pgvector `m=16`, `ef=128`: `7.465ms`
- tqvector grouped `m=16`, `ef=128`: `8.523ms`

That means tqvector’s current direct-harness latency advantage is being eaten
by end-to-end SQL/operator overhead on this measurement surface.

## Risk / Follow-up

This packet improves methodology and changes the current readout:

- the branch now has a reusable, planner-verified pgvector SQL baseline harness
- the current apples-to-apples SQL result is harsher than the direct-runtime
  result from packet `363`

The next useful follow-up is not another one-off benchmark. It is to explain
why the isolated grouped tqvector path loses its direct-latency advantage once
it is measured through SQL, likely by breaking down:

1. query encoding / input conversion overhead
2. operator / executor overhead unique to the tqvector path
3. planner or scan startup differences that do not show up in the direct
   harness
