# Review Request: C1 ADR-032 Low-Ef Binary-Prune Relaxation

## Context

Packet `297` is still the kept ADR-032 runtime base:

- binary-filtered layer-0 successors enter the frontier with approximate scores
- exact scoring is deferred until a candidate reaches the frontier head

That cut materially improved warm latency on the real `50k` seam, but low-`ef_search=40` recall
stayed at `graph_recall_at_10 = 0.8080`.

Packet `303` then tried a global-frontier exact policy. It spent much more exact work and improved
low-`ef` latency only modestly, but recall stayed flat at `0.8080`. That makes it unlikely that the
missing quality is recoverable by exact-scoring more of the same visible frontier.

## Problem

The surviving ADR-032 path still uses ADR-031-style source-local binary pruning before candidates
ever reach the visible frontier.

That is now suspicious for two reasons:

- the big ADR-032 win came from lazy exact scoring, not from binary rejection itself
- if low-`ef` recall loss is partly caused by pruning too aggressively before frontier competition,
  then exact-scoring policies later in the pipeline cannot recover those lost candidates

## Attempt

Relax or disable source-local binary pruning at low `ef_search` on top of the kept `297` exact-on-
head path.

Shape tried here:

1. keep the exact-on-head ADR-032 runtime base intact
2. only at low `ef_search` (`<= 64`), disable the source-local binary rejection budget entirely
3. remeasure the canonical warm real-`50k`, `m=8`, `ef=40` seam
4. rerun full real-`50k` recall to see whether low-`ef` quality recovers meaningfully

Concretely, the attempt changed `binary_prefilter_survivor_budget(...)` so that:

- at `ef_search > 64`, the prior ADR-031 behavior remained unchanged
- at `ef_search <= 64`, no source-local binary survivors were dropped

## Validation

This attempt was measured and then discarded. No green code checkpoint was committed from it.

All known validation reads:

- focused sanity:
  - `cargo test binary_prefilter_survivor_budget_only_filters_full_source_widths -- --exact --nocapture`: green
- release install used for measurement:
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`: green
- full `cargo test`: green
- full `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: failed
  - regression: `tests::pg_test_tqhnsw_sql_ordered_index_scan_executes`
  - failure detail: query returned `1` row instead of the requested `LIMIT 2`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`:
  - not run after the `pgrx` regression surfaced

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- invalid run, discarded:
  - launcher rejected the cell due WSL negative timing parse
  - error: `invalid negative per-query timings parsed for cached-plan: count=2 min=-984.157ms`
- valid run 1:
  - `p50=0.786ms`
  - `p95=1.080ms`
  - `p99=1.321ms`
  - `mean=0.805ms`
  - `min=0.486ms`
  - `max=1.634ms`
  - `server_qps=1241.72`
  - `wall=12.91s`
- valid run 2:
  - `p50=0.777ms`
  - `p95=1.060ms`
  - `p99=1.293ms`
  - `mean=0.796ms`
  - `min=0.501ms`
  - `max=1.719ms`
  - `server_qps=1257.03`
  - `wall=11.43s`

Reference kept `297` warm reads on the same seam:

- run 1: `p50=0.869ms`, `p99=1.559ms`, `mean=0.889ms`
- run 2: `p50=0.875ms`, `p99=1.558ms`, `mean=0.904ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.8080`
- `exact_quantized_recall_at_10 = 0.8080`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

## Outcome

Discarded.

Disabling low-`ef` source-local binary pruning did not recover any recall. It kept the low-`ef`
quality point flat at `0.8080`, even though the warm latency band improved to about
`0.796-0.805ms`.

That by itself would have made this a plausible “small latency keep.” The blocking issue is that the
full `pgrx` gate exposed a real regression in `pg_test_tqhnsw_sql_ordered_index_scan_executes`,
where the query returned fewer rows than requested.

The read from this attempt is still useful: the low-`ef` ADR-032 quality loss is not primarily
caused by source-local binary survivor pruning. The next attempt needs a different algorithmic seam.
