# Review Request: C1 ADR-030 V2 Corrected SQL Overhead With Session Reuse

## Context

Packet `366` added an executor-like slot-fetch helper and showed that heap
fetch plus simple `id` projection were not the source of the large per-query
SQL residual on the isolated grouped `m=16` lane.

That packet still left one measurement flaw in place:

- the full SQL leg of `bench_tqvector_sql_overhead_breakdown.sh` was timed in
  `per-query` mode, which opens a fresh `psql` / backend path for every timed
  query
- the internal AM, hot-path, and slot-fetch helpers were all already measured
  in a much cheaper per-cell / set-at-a-time shape

So packet `366` was still mixing two different session shapes.

## Problem

The remaining question after packet `366` was:

> is the multi-millisecond residual actually server-side, or is it mostly the
> per-query launcher / backend startup cost from the measurement path itself?

Without session reuse support in the overhead script, the branch could not
answer that cleanly in one place.

## Planned Slice

Batch the measurement fix and the corrected diagnosis together:

1. extend `bench_tqvector_sql_overhead_breakdown.sh` with the same
   `--session-mode` contract already used by the verified SQL harnesses
2. add `--timing-mode plain-server` to the overhead script so the full SQL leg
   can be timed server-side without EXPLAIN JSON parsing
3. keep the existing internal scan / hot-path / slot-fetch passes unchanged
4. rerun the isolated grouped `m=16` lane in `per-cell plain-server` mode
5. compare that corrected SQL timing directly against the internal scan total
   and the executor-like slot-fetch total

## Implementation

Updated:

- `scripts/bench_tqvector_sql_overhead_breakdown.sh`
- `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`

Concrete changes:

1. added `--session-mode` to the overhead launcher:
   - `per-query` (existing behavior, still default)
   - `per-cell` (single backend session per `ef_search` cell)
2. added `--timing-mode` to the overhead launcher:
   - `explain` (existing behavior, still default)
   - `plain-server` (server-side `clock_timestamp()` around a MATERIALIZED
     ordered query)
3. reused the existing per-cell warmup / SQL-file strategy already proven in
   the main verified SQL harnesses
4. taught the fake `psql` regression harness to understand:
   - per-cell SQL files
   - `\o` output redirection wrappers
   - plain-server timing statements
5. added a dedicated regression test that proves the overhead launcher works
   in `per-cell plain-server` mode

## Validation

Script validation:

- `bash -n scripts/bench_tqvector_sql_overhead_breakdown.sh scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh`
- `python3 -m unittest scripts.tests.test_bench_tqvector_sql_overhead_breakdown`

Observed result:

- both passed

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

So this checkpoint is green on script validation, live scratch measurements,
and clippy, but still blocked on the workstation’s existing Rust test-binary
linker issue.

## Measurements

### Corrected grouped `m=16` overhead breakdown (`per-cell plain-server`)

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
  --session-mode per-cell \
  --timing-mode plain-server \
  --output /tmp/tqvector_grouped_m16only_overhead_percell_plain_final.summary
```

Observed means:

| ef_search | SQL mean ms | internal total mean ms | executor-like total mean ms | residual SQL over internal ms | residual SQL over executor-like ms |
|----------:|------------:|-----------------------:|----------------------------:|------------------------------:|-----------------------------------:|
| 40  | `1.023` | `0.932` | `0.971` | `0.090` | `0.051` |
| 64  | `1.397` | `1.453` | `1.249` | `-0.056` | `0.148` |
| 128 | `2.116` | `2.079` | `2.063` | `0.037` | `0.052` |
| 320 | `4.305` | `4.276` | `4.316` | `0.029` | `-0.011` |

Other relevant means stayed tiny:

- `encode_mean = 0.008 / 0.008 / 0.008 / 0.008 ms`
- `slot_fetch_total_mean = 0.009 / 0.006 / 0.028 / 0.034 ms`
- `projection_mean = 0.000 ms` at every cell

## Interpretation

This packet materially changes the SQL-overhead diagnosis.

The corrected read is:

> once the SQL leg reuses a backend session and is timed server-side, the
> “missing SQL gap” essentially disappears

So packet `366` remains useful in one narrow sense:

- heap fetch and simple slot projection are still negligible

But packet `366`’s broader interpretation is superseded:

- the earlier `3.7-6.8ms` residual was **not** a strong signal of higher
  executor / SQL integration cost
- it was mostly a measurement artifact from timing the SQL leg in
  `per-query` mode while comparing it against per-cell internal helpers

On the corrected lane, full SQL mean, internal limited scan mean, and the
executor-like slot-fetch total all sit within about `0.15ms` of each other.

## Risk / Follow-up

This batch does not improve product latency directly. It fixes the measurement
surface and prevents the branch from chasing the wrong bottleneck.

Immediate implications:

1. stop treating the old per-query overhead residual as evidence of a large
   server-side SQL integration problem
2. prefer `per-cell plain-server` when comparing tqvector SQL against the
   internal scan helpers
3. when comparing tqvector versus pgvector SQL latency, make sure the session
   shape is matched before drawing architectural conclusions

The next useful work is back on product-facing operating points and honest
cross-system comparison, not on a supposed multi-millisecond tqvector SQL gap
above the AM. 
