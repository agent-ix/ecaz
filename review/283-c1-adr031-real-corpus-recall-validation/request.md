# Review Request: C1 ADR-031 Real Corpus Recall Validation

## Context

Packet `281` landed the cached ADR-031 runtime path on `main`:

- cached binary codes on graph elements
- lazy exact scoring for newly loaded graph elements
- source-local ADR-031 successor gating on the ordered-scan runtime

Packet `282` then validated the warm steady-state latency result on the
normative real `50k` lane:

- `tqhnsw_real_50k`
- `m=8`
- `ef_search=40`
- `warm-after-prime3`
- `session-mode=per-cell`
- `timing-mode=cached-plan`
- `p50 = 4.633ms`
- `p99 = 7.661ms`

That clears the `NFR-001` latency target. The next risk is quality, not more
latency shaving.

## Problem

The cached ADR-031 path makes exact scoring lazy and adds a binary-sign
approximation inside successor handling. Even though the latency result is now
strong, we still need an explicit recall/quality read on the real corpus before
treating this runtime shape as a clean keep.

The next question is:

- does ADR-031 preserve the ordered-scan result quality at `m=8`,
  `ef_search=40`
- on the real corpus lane that now meets the latency target

## Planned Investigation

First step:

- inspect the existing real-corpus recall harness and confirm whether it can
  compare the current cached ADR-031 runtime path directly against exact truth
  or against the pre-ADR-031 ordered-scan surface

Preferred scope for the first bounded read:

- real corpus
- `m=8`
- `ef_search=40`
- enough queries to detect obvious regression before committing to a long run

If the existing harness already fits, use it. If not, add the minimum launcher
or harness seam needed to make the recall comparison explicit and repeatable.

## First Bounded Read

The existing harness fits directly. I used
`tests.tqhnsw_graph_scan_recall_external_summary(...)` through the scratch
launcher with a dedicated `200`-query table:

```bash
./scripts/pg17_scratch_psql.sh --sql "
create table if not exists tqhnsw_real_50k_queries_200_adr031 as
select * from tqhnsw_real_50k_queries
order by id
limit 200;
"
```

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --index tqhnsw_real_50k_m8_idx \
  --m 8 \
  --ef-search 40 \
  --corpus-table tqhnsw_real_50k_corpus \
  --queries-table tqhnsw_real_50k_queries_200_adr031
```

Observed output:

```text
m=8
ef_search=40
corpus_rows=50000
query_count=200
graph_recall_at_10=0.8425
graph_recall_at_100=0.3992
ndcg_at_10=0.8939573
mean_abs_score_error=0.0058280844
spearman_rho_at_10=0.6464242
exact_quantized_recall_at_10=0.8425
graph_below_exact_queries=0
worst_exact_gap=0
```

Interpretation:

- the live ordered-scan graph path matched the exact quantized top-10 result on
  every sampled query (`graph_below_exact_queries=0`)
- the observed recall shortfall is therefore not from the ADR-031 runtime path;
  it is the current quantized approximation relative to brute-force fp32 truth
- for this bounded `50k` sample, ADR-031 does not introduce a new runtime
  quality regression

## Interim Keep / Pivot

Keep the cached ADR-031 runtime path.

The next question is not "does ADR-031 break graph recall versus the old live
path?" On this first bounded sample, it does not. The follow-on question is
whether that exact-quantized parity still holds on the full `1000`-query
canonical table.

## Success Criteria

- the packet records the concrete recall-validation command or harness used
- the packet records the first ADR-031 real-corpus recall comparison
- the result makes a clear keep/pivot call for the cached ADR-031 runtime path
