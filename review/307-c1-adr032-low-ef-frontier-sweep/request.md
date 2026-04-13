# Review Request: C1 ADR-032 Low-Ef Frontier Sweep

## Context

Packet `297` is the current kept ADR-032 runtime base on this branch:

- approximate layer-0 search
- lazy exact scoring at frontier consumption time

That cut materially improved warm latency, but low-`ef_search=40` recall on the full real `50k`
corpus stayed at `graph_recall_at_10 = 0.8080`.

Follow-up packets `303`, `304`, and `305` ruled out several local recovery theories:

- more exact-scoring budget in the current frontier did not help
- disabling source-local pruning did not help
- exact-reranking a wider discovered pool made both latency and recall worse

Reviewer feedback now suggests the practical next read should be an `ef_search` sweep before more
runtime redesign: redraw the current ADR-032 low-`ef` frontier on a same-build basis and see where
it overtakes the older ADR-031 quality point.

## Planned Slice

Measure the current kept ADR-032 runtime on the full real `50k` corpus at:

- `m=8`
- `ef_search = 40, 48, 56, 64`

For each cell, capture:

1. canonical warm latency
2. full real-`50k`, `1000`-query recall summary

## Validation

This was a measurement-only slice on the current kept ADR-032 runtime. No runtime code changed and
no new code checkpoint was committed.

Environment/setup used for the sweep:

- reinstalled the current branch state into the scratch cluster before measuring:
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`
- warm latency used the verified launcher with per-cell backend reuse and `warmup-passes=3`
- recall used the full `1000`-query external summary on the same build

## Measurements

Canonical warm real-`50k` latency, release build, `m=8`, `warmup-passes=3`,
`session-mode=per-cell`, `timing-mode=cached-plan`.

All known warm runs for this sweep:

- `ef=40`:
  - `p50=0.846ms`
  - `p95=1.245ms`
  - `p99=1.522ms`
  - `mean=0.877ms`
  - `min=0.517ms`
  - `max=3.237ms`
  - `server_qps=1140.82`
  - `wall=12.63s`
- `ef=48`:
  - `p50=0.903ms`
  - `p95=1.256ms`
  - `p99=1.519ms`
  - `mean=0.922ms`
  - `min=0.515ms`
  - `max=2.090ms`
  - `server_qps=1084.08`
  - `wall=13.34s`
- `ef=56`:
  - `p50=0.972ms`
  - `p95=1.301ms`
  - `p99=1.625ms`
  - `mean=0.990ms`
  - `min=0.564ms`
  - `max=2.291ms`
  - `server_qps=1010.29`
  - `wall=12.09s`
- `ef=64`:
  - `p50=1.017ms`
  - `p95=1.378ms`
  - `p99=1.664ms`
  - `mean=1.043ms`
  - `min=0.611ms`
  - `max=2.061ms`
  - `server_qps=958.96`
  - `wall=13.71s`

Full real-`50k`, `1000` queries.

All known recall rows for this sweep:

- `ef=40`:
  - `graph_recall_at_10 = 0.8080`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 0`
  - `worst_exact_gap = 0`
- `ef=48`:
  - `graph_recall_at_10 = 0.8255`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 1`
  - `worst_exact_gap = 1`
- `ef=56`:
  - `graph_recall_at_10 = 0.8417`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 3`
  - `worst_exact_gap = 1`
- `ef=64`:
  - `graph_recall_at_10 = 0.8519`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 8`
  - `worst_exact_gap = 1`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

Useful comparison points from the earlier branch history:

- kept ADR-032 packet `297` at `ef=40`: `mean ~= 0.889-0.904ms`, `graph_recall_at_10 = 0.8080`
- ADR-031 Tier 1 packets `283` and `287` at `ef=40`: `graph_recall_at_10 ~= 0.8397-0.8428`

## Outcome

The sweep produced a practical low-`ef` frontier.

Important reads:

- `ef=56` is the first same-build ADR-032 point that roughly matches the old ADR-031 `ef=40`
  recall band (`0.8417` vs `~0.8397-0.8428`) while staying just under `1ms` mean (`0.990ms`).
- `ef=64` pushes recall higher to `0.8519`, but crosses just above the `1ms` mean line
  (`1.043ms`).
- `ef=48` is still firmly sub-`1ms`, but recall only rises to `0.8255`, which is better than
  `ef=40` but still below the older ADR-031 `ef=40` quality point.

So the current kept ADR-032 runtime already has a credible practical operating point:

- `m=8`, `ef_search=56` for “about `1ms`, about `0.842` recall”
- `m=8`, `ef_search=64` for “just over `1ms`, about `0.852` recall”

This does not settle the larger question of whether ADR-032 can reach `>0.90` recall near `1ms`,
but it does show that a moderate `ef_search` increase materially improves the latency/recall
frontier without more runtime redesign.

## Next Step

The next runtime experiment should still target trajectory correction rather than more rerank:

- exact-scored multi-seed anchoring

That is the highest-signal remaining ADR-032 structural experiment before a larger pivot toward a
true ADR-030 redesign.

- the packet records all known warm and recall results for `ef=40/48/56/64`
- the read is same-build and apples-to-apples across the sweep
- if a practical low-`ef` operating point appears, the packet calls it out explicitly
