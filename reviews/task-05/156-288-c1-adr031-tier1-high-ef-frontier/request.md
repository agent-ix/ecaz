# Review Request: C1 ADR-031 Tier 1 High-EF Frontier

## Context

Packet `287` established that the ADR-031 Tier 1 inline cache slice is a major
warm-latency win on the real `50k` canonical seam at:

- `m=8`
- `ef_search=40`
- `1000` queries
- `warm-after-prime3`
- `session-mode=per-cell`
- `timing-mode=cached-plan`

with repeated release reads around:

- `p50 ~= 1.48ms`
- `p99 ~= 2.4ms`
- `mean ~= 1.51ms`

and a current-build recall recheck showing no runtime regression versus exact
quantized results on that same `ef_search=40` seam.

The next question is not whether ADR-031 works at all. It does. The next
question is where the current latency/recall frontier sits at the higher
`ef_search` settings that matter for apples-to-apples comparison with the older
A4 gate work.

## Problem

The old A4 evidence and the current ADR-031 Tier 1 evidence are easy to
miscompare because they use different seams:

- A4 closeout centered on `ef_search=128` and smaller real-query slices
- packet `287` centered on `ef_search=40` and the full canonical `1000`-query
  table

We need two explicit reads on the current Tier 1 build:

1. canonical current-build recall + latency at `ef_search=128` and `200`
2. a same-query-table apples-to-apples read against the old A4 `queries_50`
   surface so the comparison does not mix query counts or `ef_search`

## Planned Investigation

On the current Tier 1 build:

1. Run full real-`50k` canonical recall summaries at:
   - `m=8`, `ef_search=128`
   - `m=8`, `ef_search=200`
2. Run full real-`50k` warm latency summaries at the same two points.
3. Reuse the historical `tqhnsw_real_50k_queries_50` table and record the
   current `m=8` high-`ef_search` recall there as the apples-to-apples A4
   comparison seam.

## Success Criteria

- the packet records current-build real-`50k` recall at `ef_search=128` and
  `200`
- the packet records current-build real-`50k` warm latency at `ef_search=128`
  and `200`
- the packet records a same-query-table comparison against the old
  `queries_50` A4 surface
- the packet makes a clear call on whether Tier 2 should be next or whether the
  high-`ef_search` frontier still needs work first

## Current Canonical Recall

Commands used on the current Tier 1 build:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 128 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 200 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries
```

Observed outputs:

```text
8  128  50000  1000  0.8977  0.85971  0.9341158  0.006018223   0.77886665  0.8428  9  1
8  200  50000  1000  0.9039  0.88845  0.938355   0.006021826   0.79619354  0.8428  9  1
```

Readout:

- canonical full-table graph Recall@10 is now `0.8977` at `ef=128`
- canonical full-table graph Recall@10 is now `0.9039` at `ef=200`
- both remain above the current exact-quantized oracle on the same build
  (`0.8428`), so this is still not a simple live-vs-oracle regression story

## Current Canonical Warm Latency

Commands used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 128 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output /tmp/adr031_tier1_real_50k_m8_ef128_warm.summary
```

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output /tmp/adr031_tier1_real_50k_m8_ef200_warm.summary
```

Observed outputs:

```text
ef_search=128
p50=3.416ms
p95=4.462ms
p99=5.227ms
mean=3.409ms
```

```text
ef_search=200
p50=4.852ms
p95=6.352ms
p99=7.076ms
mean=4.772ms
```

So the current Tier 1 frontier at higher `ef_search` is still fast:

- `ef=128`: well below `5ms` mean and p50
- `ef=200`: still below `5ms` mean and only slightly below `5ms` p50

## Apples-to-Apples A4 Slice

The old A4 records used the staged `50`-query real table, but the current
scratch fixture did not have that table loaded. I recreated it with the same
shape as the earlier A4 packets:

```sql
create table if not exists tqhnsw_real_50k_queries_50 as
select * from tqhnsw_real_50k_queries
order by id
limit 50;
```

Then I reran the current Tier 1 build on that same query-count seam:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 128 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 200 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_50
```

Observed current-build outputs:

```text
8  128  50000  50  0.89   0.8734  0.92889374  0.0055691516  0.7583029   0.86  1  1
8  200  50000  50  0.894  0.8944  0.9313327   0.0055674054  0.76424235  0.86  1  1
```

Historical A4 references from packet `225` on the same `50`-query gate slice:

```text
8   128  0.944  0.89 t
8   200  0.948       t
```

## Readout

The apples-to-apples comparison says the current high-`ef_search` frontier is
not as strong as the older A4 `50`-query gate slice:

- `queries_50`, `ef=128`: `0.944 -> 0.890`
- `queries_50`, `ef=200`: `0.948 -> 0.894`

At the same time, the current canonical full-table frontier is still viable:

- `ef=128`: `0.8977` recall at `3.409ms` mean
- `ef=200`: `0.9039` recall at `4.772ms` mean

So the current Tier 1 ADR-031 build is fast enough at higher `ef_search`, but
the quality frontier no longer looks equivalent to the earlier A4-era slice.

That means Tier 2 should **not** be treated as the automatic next step yet.
Before more ADR-031 optimization work, the right next question is isolation:

1. compare the current build with ADR-031 enabled vs disabled on the same
   high-`ef_search` recall seams
2. determine whether the high-`ef_search` drop is actually from ADR-031 or from
   later non-ADR-031 evolution in the graph/runtime/index state

If that A/B shows ADR-031 is innocent, Tier 2 is fine to pursue next. If it
shows ADR-031 is the cause, Tier 2 is the wrong next move.
