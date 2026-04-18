# Review Request: C1 ADR-030 V2 Real-50k M16 Current-Head Validation

Current head at execution: `6b4eac0`

## Context

Packet `405` already captured the current-head task-15 gate pass on the
canonical `50k` lane:

- explicit `pq_fastscan`
  - `m=8`: `0.8231`, `0.9078`, `0.9174`
  - `m=16`, `ef=200`: `0.9671`
- explicit `turboquant`
  - `m=8`: `0.8301`, `0.8927`, `0.9011`
  - `m=16`, `ef=200`: `0.9376`

But there was still one exact question left open in the branch narrative:

- had `m=16` been rerun on the current head at the full `1000`-query
  `ef_search=128` summary lane, not just the gate lane or the older
  `queries_50` packets (`362`)?

That is the gap this packet closes.

## Problem

The branch already had strong historic `m=16` evidence:

- packet `362`: `pq_fastscan` `50k`, `m=16`, `queries_50`, `ef=128` reached
  `0.936`
- packet `405`: current-head `50k` gate showed `m=16`, `ef=200` at `0.9671`
  for `pq_fastscan` and `0.9376` for `turboquant`

But it did not yet have a fresh current-head full-summary rerun for:

- `m=16`
- `ef_search=128`
- full `1000`-query `tqhnsw_real_50k_queries`
- canonical explicit-format index families

## Planned Slice

No code change. Run the missing current-head summaries explicitly against the
live `~/.pgrx` cluster and record the results.

Commands:

```bash
TQV_PG_SOCKET_DIR=/home/peter/.pgrx ./scripts/run_real_corpus_recall_scratch.sh summary \
  --prefix tqhnsw_real_50k \
  --storage-format pq_fastscan \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_queries

TQV_PG_SOCKET_DIR=/home/peter/.pgrx ./scripts/run_real_corpus_recall_scratch.sh summary \
  --prefix tqhnsw_real_50k \
  --storage-format turboquant \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_queries
```

## Results

Current-head `pq_fastscan`, `m=16`, `ef=128`, `50k`, `1000` queries:

- `graph_recall_at_10 = 0.9635`
- `graph_recall_at_100 = 0.76542`
- `ndcg_at_10 = 0.976963`
- `mean_abs_score_error = 0`
- `spearman_rho_at_10 = 0.92972225`
- `exact_quantized_recall_at_10 = 0.9144`
- `graph_below_exact_queries = 98`
- `worst_exact_gap = 3`

Artifact:

- `tmp/real_corpus_runs/20260417T012221Z_summary_tqhnsw_real_50k_pq_fastscan_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

Current-head `turboquant`, `m=16`, `ef=128`, `50k`, `1000` queries:

- `graph_recall_at_10 = 0.9342`
- `graph_recall_at_100 = 0.93023`
- `ndcg_at_10 = 0.9594091`
- `mean_abs_score_error = 0.0060423133`
- `spearman_rho_at_10 = 0.90292054`
- `exact_quantized_recall_at_10 = 0.9144`
- `graph_below_exact_queries = 80`
- `worst_exact_gap = 2`

Artifact:

- `tmp/real_corpus_runs/20260417T012221Z_summary_tqhnsw_real_50k_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

## Outcome

The answer to "have we tested `m=16` yet?" is now exact on the pushed head:

1. yes, `m=16` is covered on the gate lane from packet `405`
2. yes, `m=16` is now also covered on the full `1000`-query summary lane at
   `ef_search=128`
3. on that lane, `pq_fastscan` remains ahead of `turboquant` on
   `graph_recall_at_10`

This materially strengthens the "first-class on main" story because the strong
`m=16` result is no longer only a historical `queries_50` packet or a
single-cell gate check.

## Next Slice

Use the refreshed current-head evidence plus packet `405` to drive merge review
and any final task-15 closeout, unless new reviewer feedback lands first.
