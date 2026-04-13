# Review Request: C1 ADR-032 Exact-Scored Multi-Seed Anchoring

## Context

Packet `307` established the practical low-`ef` ADR-032 frontier on the current kept runtime:

- `ef=56`: `graph_recall_at_10 = 0.8417`, `mean = 0.990ms`
- `ef=64`: `graph_recall_at_10 = 0.8519`, `mean = 1.043ms`

That is useful operationally, but it does not remove the underlying low-`ef` trajectory gap:

- kept ADR-032 at `ef=40` still sits at `graph_recall_at_10 = 0.8080`
- post-discovery fixes (`303`, `304`, `305`) did not recover that gap

Reviewer feedback now points at the next structural seam: change the *early expansion trajectory*
rather than spending more exact work later.

## Problem

The current ADR-032 layer-0 search starts from a very narrow seed situation:

- one upper-layer descent winner
- then approximate-scored layer-0 exploration from there

If that first anchor is slightly wrong, a low `ef_search` budget can spend most of its expansions in
the wrong neighborhood. Later exact scoring cannot recover candidates that were never discovered.

## Attempt

Prototype exact-scored multi-seed anchoring for low-`ef` ADR-032 scans.

Likely first cut:

1. derive a small upper-layer seed set rather than a single seed
2. exact-score only that small seed set
3. start the existing cheap approximate layer-0 search from those exact-scored seeds
4. leave the rest of the ADR-032 runtime path unchanged

Concrete first cut used here:

- activation only for binary low-`ef_search <= 64`
- anchor count `4`
- derive a small upper-layer seed set instead of a single greedy-descent winner
- exact-score those anchor seeds, then feed them into the existing approximate layer-0 search

## Validation

This attempt was measured and then discarded. No green code checkpoint was committed from it.

All known validation reads:

- focused sanity:
  - `cargo test adr032_exact_seed_anchor_limit_only_arms_low_ef_binary_scans -- --exact --nocapture`: green
- release install used for measurement:
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config --features 'pg17 pg_test' --no-default-features`: green
- no full `cargo test` / `cargo pgrx test` / clippy gate was run after the first measurement turned clearly negative

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- valid run 1:
  - `p50=0.820ms`
  - `p95=1.083ms`
  - `p99=1.276ms`
  - `mean=0.835ms`
  - `min=0.534ms`
  - `max=1.406ms`
  - `server_qps=1196.93`
  - `wall=11.63s`

Reference current kept ADR-032 packet `307` / `297` warm reads on the same seam:

- `mean=0.877ms` on the same-build `307` sweep
- prior kept `297` runs: `mean ~= 0.889-0.904ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.7827`
- `exact_quantized_recall_at_10 = 0.7827`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

## Outcome

Discarded.

This cut did change the early trajectory, but in the wrong direction:

- warm latency improved from the kept `ef=40` band to `mean=0.835ms`
- recall fell from `0.8080` to `0.7827`

So exact-scored multi-seed anchoring, at least in this simple “4 exact seeds before approximate
layer-0 search” form, does not recover the low-`ef` quality gap. It makes the search faster by
steering it more aggressively, but it steers it toward worse neighborhoods on this corpus.
