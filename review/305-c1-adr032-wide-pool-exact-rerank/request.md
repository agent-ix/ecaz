# Review Request: C1 ADR-032 Wide-Pool Exact Rerank

## Context

Packet `297` is still the kept ADR-032 runtime base:

- binary-scored candidates drive the low-cost layer-0 search
- exact scoring is deferred instead of being paid eagerly on cache miss

That cut shifted the warm latency frontier decisively, but low-`ef_search=40` recall stayed at
`graph_recall_at_10 = 0.8080`.

Follow-up packets `303` and `304` ruled out two simpler explanations:

- `303`: exact-scoring more of the existing visible frontier did not recover recall
- `304`: relaxing low-`ef` source-local binary pruning did not recover recall either

That points at a different failure mode: the approximate search may be discovering the right
candidates, but the final exact-rerank pool is too narrow because it is effectively capped at the
same `ef_search` window.

## Problem

The current ADR-032 path still stages only the approximate search result window of size `ef_search`
for later exact adjudication.

If the approximate search is finding good nodes just outside that final `ef_search` frontier, then:

- exact-scoring the current frontier more carefully cannot help
- relaxing source-local pruning cannot help
- the next lever is a bounded *wider* cheap candidate pool followed by exact rerank

## Attempt

Prototype a two-stage low-`ef` ADR-032 search:

1. run the existing cheap approximate layer-0 search
2. keep a bounded candidate pool larger than `ef_search`
3. exact-score that widened pool after search
4. stage the best exact-scored candidates for output

Likely first cut:

- only arm for binary low-`ef_search <= 64`
- widen the rerank pool to `2x ef_search`
- keep the rest of the runtime path unchanged

Concretely, the first cut used:

- `ef_search <= 64` activation
- rerank pool width `min(discovered_count, ef_search * 2)`
- approximate layer-0 search unchanged
- exact rerank applied to the widened discovered-candidate pool before staging outputs

## Validation

This attempt was measured and then discarded. No green code checkpoint was committed from it.

All known validation reads:

- focused sanity:
  - `cargo test adr032_wide_rerank_pool_only_arms_low_ef_binary_scans -- --exact --nocapture`: green
  - `cargo test binary_prefilter_survivor_budget_only_filters_full_source_widths -- --exact --nocapture`: green
- release install used for measurement:
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`: green
- no full `cargo test` / `cargo pgrx test` / clippy gate was run after the measurement cut turned clearly negative

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- valid run 1:
  - `p50=0.918ms`
  - `p95=1.128ms`
  - `p99=1.380ms`
  - `mean=0.923ms`
  - `min=0.622ms`
  - `max=1.626ms`
  - `server_qps=1083.60`
  - `wall=13.51s`

Reference kept `297` warm reads on the same seam:

- run 1: `p50=0.869ms`, `p99=1.559ms`, `mean=0.889ms`
- run 2: `p50=0.875ms`, `p99=1.558ms`, `mean=0.904ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.7790`
- `exact_quantized_recall_at_10 = 0.7790`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

## Outcome

Discarded.

Widening the exact-rerank pool after the cheap approximate search moved in the wrong direction on
both axes:

- warm latency regressed from the kept `297` `~0.889-0.904ms` band to `0.923ms`
- full real-`50k` low-`ef` recall fell from `0.8080` to `0.7790`

This is a strong negative result. It means the low-`ef` ADR-032 quality point is not recoverable by
simply exact-reranking more of the approximate search output. On this branch, that wider exact
rerank only pushes the results further toward the weaker quantized-exact operating point.
