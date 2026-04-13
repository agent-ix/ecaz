# Review Request: C1 ADR-032 Exact-On-Head Frontier Promotion

## Context

Retrospective split from the original packet `293`.

Packets `294` through `296` established that the next viable ADR-032 lever was exact-score timing,
not more cache-shape substitution or more minor survivor pruning.

## Attempt

- admit binary-filtered layer-0 successors to the frontier with approximate scores
- stop exact-scoring them immediately during source expansion
- exact-score a candidate only when it reaches the frontier head
- if exact scoring makes it worse than the next queued candidate, requeue it with its exact score
  and continue

This changed the exact-score lifecycle from "score survivors eagerly" to "score at frontier head."

## Validation

Green code checkpoint:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurements

Warm canonical real-`50k` latency, release build, `m=8`, `warmup-passes=3`,
`session-mode=per-cell`, `timing-mode=cached-plan`.

Reference kept ADR-031 Tier 1 means:

- `ef=40`: `mean ~= 1.507-1.510ms`
- `ef=128`: `mean = 3.409ms`
- `ef=200`: `mean = 4.772ms`

All known warm runs for this attempt:

- `ef=40` run 1: `p50=0.869ms`, `p99=1.559ms`, `mean=0.889ms`
- `ef=40` run 2: `p50=0.875ms`, `p99=1.558ms`, `mean=0.904ms`
- `ef=128`: `p50=1.643ms`, `p99=2.420ms`, `mean=1.657ms`
- `ef=200`: `p50=2.363ms`, `p99=3.509ms`, `mean=2.380ms`

Full real-`50k`, `1000` queries.

Important note: on this branch, `exact_quantized_recall_at_10` is not a fully trustworthy exact
reference because the comparison SQL can itself route through the live tqhnsw index. The
meaningful quality read is `graph_recall_at_10` versus fp32 truth. The packet still records the
other fields because they were part of the original measurement surface.

All known recall rows for this attempt:

- `ef=40`:
  - `graph_recall_at_10 = 0.8080`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 0`
  - `worst_exact_gap = 0`
- `ef=128`:
  - `graph_recall_at_10 = 0.8861`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 12`
  - `worst_exact_gap = 1`
- `ef=200`:
  - `graph_recall_at_10 = 0.8968`
  - `exact_quantized_recall_at_10 = 0.8080`
  - `graph_below_exact_queries = 13`
  - `worst_exact_gap = 1`

## Outcome

Kept. This is the current ADR-032 runtime cut on the branch.

This was the first ADR-032 slice that decisively beat the kept ADR-031 path on the real-`50k`
warm surface. The tradeoff is that low-`ef_search=40` recall is materially lower, while the
latency/recall frontier at `ef=128` and `ef=200` is much better than the prior path.
