# Review Request: C1 ADR-030 V2 Staged 1k/10k/50k Runtime Validation

## Context

Packet `351` landed the first live grouped-v2 runtime cutover behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- grouped approximate scan scoring
- live rerank window `4`
- preserved approximate-rank sidecars for diagnostics
- grouped comparison and window diagnostics still anchored to baseline order

That made the next question operational rather than structural:

- does the live grouped-v2 path produce a promising operating point on real
  corpus data
- and is it worth pushing directly to wider `50k` SQL benches, or should the
  next slice return to runtime tuning first

The user also asked explicitly whether this branch had been validated at
smaller scales (`1k`, `10k`) before treating the `50k` lane as authoritative.

## Problem

Packet `351` proved correctness on pg fixtures, but not whether the new
grouped-v2 runtime shape is directionally good on staged real-corpus data.

Before spending more time on broader `50k` SQL benchmarking, we needed:

1. a small-scale sanity check (`1k`)
2. a medium-scale isolated runtime read (`10k`)
3. the first full-lane grouped-vs-scalar comparison on the canonical `50k`
   corpus

## Measurement Setup

Scratch-cluster setup:

- restarted the scratch `pg17` cluster with:
  - `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
  - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
- installed the current branch into the scratch cluster with:

```bash
PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test \
  --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config \
  --features 'pg17 pg_test' --no-default-features
```

Operational note:

- the scratch database already had a same-version `pg_test` extension install,
  so new SQL wrappers were not auto-created in the database
- I did not drop and recreate the extension because that would have destroyed
  the loaded real-corpus fixtures
- existing recall surfaces in schema `tests` were sufficient for this packet

Isolated grouped prefixes:

- `tqhnsw_real_1k_grouped_*`
  - derived from the first `1000` rows of `tqhnsw_real_10k_corpus`
  - used the full `200`-query `tqhnsw_real_10k_queries` table
- `tqhnsw_real_10k_grouped_*`
  - full copy of `tqhnsw_real_10k_{corpus,queries}`
  - built a dedicated grouped index so the planner would not choose the scalar
    sibling index by accident
- `tqhnsw_real_50k_grouped_m8_idx`
  - grouped-v2 `m=8` index on the already-loaded canonical `50k` corpus

Recall/latency harness:

```sql
select * from tests.tqhnsw_graph_scan_recall_ef_sweep(
  '<corpus_table>',
  '<query_table>',
  '<index_name>',
  8,
  array[40,64,100,128,160,200]
)
order by ef_search;
```

Planner-facing SQL latency read:

```bash
scripts/bench_sql_latency_scratch.sh \
  --prefix <prefix> \
  --m 8 \
  --ef-search 40,64,100,128,160,200 \
  --query-limit 50 \
  --cache-state cold
```

For `50k`, I only ran a narrow planner probe at `ef_search=128`, `query_limit=10`
to avoid spending another long batch before we knew whether the grouped lane was
actually competitive.

## Results

### 1k grouped subset

`tests.tqhnsw_graph_scan_recall_ef_sweep('tqhnsw_real_1k_grouped_corpus', 'tqhnsw_real_1k_grouped_queries', 'tqhnsw_real_1k_grouped_m8_idx', 8, array[40,64,100,128,160,200])`

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | mean latency ms |
|-----------|-------------------|---------------------------|-----------------|
| 40 | 0.8685 | 0.9555 | 0.9337 |
| 64 | 0.8890 | 0.9555 | 1.1736 |
| 100 | 0.9090 | 0.9555 | 1.4535 |
| 128 | 0.9120 | 0.9555 | 1.7371 |
| 160 | 0.9135 | 0.9555 | 1.9800 |
| 200 | 0.9155 | 0.9555 | 2.1669 |

Interpretation:

- useful smoke validation only
- grouped-v2 is stable, but this subset is not a compelling operating point and
  does not reach the exact-quantized ceiling

### 10k grouped vs scalar

Grouped isolated prefix:

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.9245 | 0.9245 | 0.8661 |
| 64 | 0.9320 | 0.9245 | 1.1973 |
| 100 | 0.9325 | 0.9245 | 1.6380 |
| 128 | 0.9350 | 0.9245 | 2.0629 |
| 160 | 0.9355 | 0.9245 | 2.2870 |
| 200 | 0.9360 | 0.9245 | 2.6838 |

Scalar baseline:

| ef_search | scalar Recall@10 | exact-quantized Recall@10 | scalar mean latency ms |
|-----------|------------------|---------------------------|------------------------|
| 40 | 0.9310 | 0.9310 | 2.6765 |
| 64 | 0.9335 | 0.9310 | 3.6525 |
| 100 | 0.9385 | 0.9310 | 5.1261 |
| 128 | 0.9400 | 0.9310 | 6.0313 |
| 160 | 0.9400 | 0.9310 | 6.9815 |
| 200 | 0.9400 | 0.9310 | 8.1569 |

Planner-facing SQL latency on the isolated `10k` grouped prefix:

| ef_search | grouped mean ms | scalar mean ms |
|-----------|-----------------|----------------|
| 40 | 4.456 | 5.319 |
| 64 | 4.826 | 6.418 |
| 100 | 5.418 | 8.087 |
| 128 | 5.850 | 9.256 |
| 160 | 6.172 | 10.283 |
| 200 | 6.735 | 11.550 |

Interpretation:

- on `10k`, grouped-v2 is directionally good
- recall trails scalar by only about `0.5-0.7` points at the top end
- direct-harness latency is roughly `2-3x` better across the sweep
- planner-facing SQL latency is also consistently better on the isolated grouped
  prefix

### 50k grouped vs scalar

Grouped `50k` sweep:

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.8560 | 0.8560 | 1.6039 |
| 64 | 0.8620 | 0.8560 | 2.1713 |
| 100 | 0.8680 | 0.8560 | 3.0108 |
| 128 | 0.8700 | 0.8560 | 3.3638 |
| 160 | 0.8740 | 0.8560 | 4.0595 |
| 200 | 0.8760 | 0.8560 | 4.6176 |

Scalar `50k` baseline:

| ef_search | scalar Recall@10 | exact-quantized Recall@10 | scalar mean latency ms |
|-----------|------------------|---------------------------|------------------------|
| 40 | 0.8600 | 0.8560 | 4.0557 |
| 64 | 0.8760 | 0.8560 | 2.2890 |
| 100 | 0.8840 | 0.8560 | 2.9891 |
| 128 | 0.8900 | 0.8560 | 3.3630 |
| 160 | 0.8920 | 0.8560 | 4.0729 |
| 200 | 0.8940 | 0.8560 | 4.4875 |

Planner-facing SQL probe at `ef_search=128`, `query_limit=10`:

| path | mean ms | p50 ms | p95 ms |
|------|---------|--------|--------|
| grouped `tqhnsw_real_50k_grouped_m8_idx` | 8.996 | 8.940 | 10.622 |
| scalar `tqhnsw_real_50k_m8_idx` | 8.457 | 8.611 | 9.327 |

Interpretation:

- `50k` is the important result, and it is not yet a grouped-v2 win
- grouped recall is below scalar recall across the sweep
- grouped direct-harness latency is only clearly better at the first low-`ef`
  cell, and the planner-facing `ef=128` probe is slightly worse than scalar
- by the time grouped recall reaches `0.870-0.876`, scalar is still both
  higher-recall and roughly equal or slightly better on latency

## Keep / Pivot

Keep the grouped-v2 runtime lane, but do not treat packet `351` as the
beginning of a `50k` benching victory.

The staged validation says:

1. the grouped-v2 runtime path is real and promising on `10k`
2. the same shape does not yet carry to a convincing `50k` operating point
3. the next ADR-030 slice should go back to runtime/pipeline tuning, not widen
   SQL benchmarking first

## Next Step

The next narrow runtime batch should target the `50k` gap directly. Most likely
follow-on directions are:

1. inspect whether the live `window=4` rerank cutover is too small or too late
   for the larger lane
2. wire more of the intended `binary -> grouped -> rerank` pipeline explicitly
   instead of treating grouped scoring as a mostly standalone scan replacement
3. re-measure the `50k` seam after that narrower runtime change before spending
   time on a full `50k` SQL latency matrix

## Success Criteria

- the packet records explicit staged runtime validation at `1k`, `10k`, and
  `50k`
- the packet records at least one planner-facing SQL bench on an isolated
  grouped prefix
- the packet makes an explicit keep/pivot call for ADR-030 after the first real
  grouped-v2 corpus read
