# Review Request: C1 ADR-032 Periodic In-Search Trajectory Correction

## Context

Packet `307` established the current practical ADR-032 operating points on the kept runtime:

- `ef=56`: `graph_recall_at_10 = 0.8417`, `mean = 0.990ms`
- `ef=64`: `graph_recall_at_10 = 0.8519`, `mean = 1.043ms`

That is useful, but it leaves the lower-`ef` structural gap unresolved:

- kept `ef=40`: `graph_recall_at_10 = 0.8080`

Packet `308` tested exact-scored multi-seed anchoring as the first trajectory-oriented follow-up.
That moved in the wrong direction:

- latency improved to `mean = 0.835ms`
- recall dropped to `0.7827`

So the simple “better initial seeds” version is not enough.

## Problem

The remaining reviewer-suggested structural seam is to correct the search trajectory *during*
low-`ef` layer-0 exploration, not only before it starts and not only after candidates have already
been discovered.

The aim is:

- keep the cheap approximate search as the default driver
- periodically exact-score the would-be next expansion source
- if the exact score says that source is worse than the next approximate frontier option, requeue it
  with its exact score and expand a different source instead

That changes which graph neighborhoods get explored under low `ef_search`, which is the remaining
hypothesis not yet tested directly.

## Attempt

Prototype periodic in-search trajectory correction for binary low-`ef` ADR-032 scans.

Concrete first cut used here:

1. only arm for binary scans with `ef_search <= 64`
2. during layer-0 search, exact-score every `4th` would-be expansion source
3. compare that exact score against the current approximate frontier head
4. if exact scoring makes it worse, requeue it and expand a different source instead
5. leave the rest of the ADR-032 runtime path unchanged

## Validation

This attempt was measured and then discarded. No green code checkpoint was committed from it.

All known validation reads:

- focused sanity:
  - `cargo test adr032_trajectory_correction_period_only_arms_low_ef_binary_scans -- --exact --nocapture`: green
- release install used for measurement:
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`: green
- no full `cargo test` / `cargo pgrx test` / clippy gate was run after the first measurement turned decisively negative

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- valid run 1:
  - `p50=2.249ms`
  - `p95=3.331ms`
  - `p99=3.936ms`
  - `mean=2.359ms`
  - `min=1.445ms`
  - `max=6.654ms`
  - `server_qps=423.95`
  - `wall=17.91s`

Reference current kept ADR-032 packet `307` / `297` warm reads on the same seam:

- same-build `307` sweep: `mean=0.877ms`
- prior kept `297` runs: `mean ~= 0.889-0.904ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.3111`
- `exact_quantized_recall_at_10 = 0.3111`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

## Outcome

Discarded.

This cut is not a frontier tradeoff. It is simply the wrong direction.

- warm latency regressed badly, from the kept `ef=40` ADR-032 band (`~0.877-0.904ms`) to
  `mean=2.359ms`
- full real-`50k` recall collapsed from `0.8080` to `0.3111`

So periodic in-search trajectory correction in this simple “exact-score every `4th` would-be
expansion source and requeue if it falls behind the approximate head” form should not be pursued
further. It spends exact work at the wrong time and destabilizes the layer-0 search trajectory
instead of improving it.

The runtime code from this attempt was restored after measurement.

## Next Step

This was the last high-signal ADR-032 structural experiment on the current search shape.

The remaining practical options are now:

- keep the existing ADR-032 frontier as-is (`ef=56` / `ef=64` from packet `307`)
- or pivot to ADR-030 as the next larger redesign lane
