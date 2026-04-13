# Review Request: C1 ADR-032 Node-Centric Redesign Reframe

## Context

ADR-031 is already very promising on the current persisted format, so ADR-032 is being
reframed as the larger runtime redesign lane that tries to compound that win without taking
on ADR-030's index-v2 encoding/layout work yet.

Packets `291` and `292` are now explicitly treated as rejected weak variants:

- `291`: element-cache arena only
- `292`: neighbor-cache arena only

Both validated and both regressed the canonical warm real-`50k` seam. That means ADR-032 is
not "replace one cache object at a time." It needs to be a broader scan-architecture change.

## Reframed Goal

Make the graph-first scan algorithmically cheaper by turning scan-local hot state into one
node-centric lifecycle instead of three independent tid-keyed caches.

Current split state:

- element cache keyed by `element_tid`
- neighbor cache keyed by `neighbortid`
- exact score cache keyed by `element_tid`

Target state:

- one fused `CachedGraphNode` slot keyed by `element_tid`
- lazy adjacency fill on that same slot
- exact-score lifecycle attached to that same slot
- frontier/result bookkeeping carried as slot ids, not repeated tids

## Planned First Real Slice

The next ADR-032 implementation slice should:

1. introduce `ScanNodeArena`
2. introduce `CachedGraphNode { element, neighbors: Option<...>, exact_score_state, ... }`
3. remove the separate scan-local neighbor cache
4. switch source expansion to one node-slot lookup plus lazy adjacency fill
5. preserve current semantics before attempting deeper frontier/result slot conversion

## Attempted First Cut: Fused Element-Local Neighbor/Score State

I tried the lightest plausible first cut before moving all the way to slot-based traversal:

- remove the separate scan-local neighbor cache
- remove the separate exact-score cache
- attach lazy neighbors and exact-score-once state directly to `CachedGraphElement`
- keep the rest of traversal keyed by tids

This was a legitimate step beyond packets `291` and `292`, but it still stopped short of the
actual ADR-032 target because frontier and result bookkeeping continued to carry tids rather
than stable scan-local node ids.

## Measurement

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- standing ADR-031 Tier 1 baseline: `mean ~= 1.507-1.510ms`
- first fused-node cut run 1: `p50=1.560ms`, `p99=2.525ms`, `mean=1.588ms`
- first fused-node cut run 2: `p50=1.564ms`, `p99=2.720ms`, `mean=1.602ms`

Validation for the attempted cut was green before benchmarking:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Conclusion

This first cut is another rejected weak variant.

It does co-locate more node-local state than packets `291` and `292`, but it still leaves the
scan algorithm operating on tids rather than stable node slots. That means it does not remove
enough repeated lookup/join work to justify the extra interior mutability and object shape.

The runtime code for this cut was discarded after the two confirmation runs above.

## Updated Next Slice

The next legitimate ADR-032 implementation must move past "fused cache entry" and into the
actual redesign seam:

1. introduce stable scan-local node slots
2. let frontier/result bookkeeping carry slot ids instead of tids
3. make source expansion and exact-score lifecycle operate through those slots
4. only then judge whether ADR-032 is reducing repeated node-state churn in a way that can beat
   the kept ADR-031 baseline

## Attempted Second Cut: Slot-Based Frontier + Bootstrap Scheduler

I then implemented the first real slot-based ADR-032 slice:

- introduce a scan-local `ScanNodeArena`
- keep one cached node object per `element_tid`
- move frontier and bootstrap scheduler storage to node-slot ids internally
- keep the test/readout helpers projecting slots back to tids so the existing review surface
  stays readable

This is the first slice that actually exercised the intended ADR-032 architecture instead of
 another cache-shape substitution.

## Validation

The slot-based cut was fully green before measurement:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurement: Slot-Based Frontier Cut

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- standing ADR-031 Tier 1 baseline: `p50~=1.480-1.485ms`, `mean~=1.507-1.510ms`
- slot-based cut run 1: `p50=1.548ms`, `p99=2.636ms`, `mean=1.592ms`
- slot-based cut run 2: `p50=1.542ms`, `p99=2.623ms`, `mean=1.579ms`

So the first actual slot-based traversal cut is still slower than the kept ADR-031 Tier 1
baseline by about `0.07-0.09ms`.

## Conclusion: Stronger Than The First Cut, Still Not A Keep

This second cut is stronger than the earlier fused-node-cache discard because it really does
move frontier and scheduler state onto stable node slots. But it still does not beat the
kept ADR-031 path on the canonical warm seam.

The runtime code for this slot-based cut was also discarded after the confirmation rerun.

## Current Read

ADR-032 is not dead, but the evidence is now sharper:

- cache-shape substitutions alone do not help
- slot-based frontier bookkeeping alone also does not help enough
- if ADR-032 is going to matter, the next slice must reduce expensive work per traversal step,
  not just where the state is stored

The likely next legitimate ADR-032 question is no longer "can slots replace tids?" but
"can a slot-centric scan state reduce exact-score pressure or avoid redundant source expansion
in a way the current ADR-031 path does not?"

## Draft Measurement: ADR-031 On/Off Score-Pressure Read

Before attempting another ADR-032 runtime cut, I sampled the existing debug hot-path counters on
the kept ADR-031 branch to see whether the next redesign should be chasing exact-score volume or
something else.

Method:

- real `50k` fixture
- `m=8`, `ef_search=40`
- first `20` queries from `tqhnsw_real_50k_queries`
- `tests.tqhnsw_debug_scan_hot_path_profile(...)` on the current kept path
- compared with `tqhnsw.disable_binary_prefilter` reset vs `on`

Readout:

- ADR-031 enabled:
  - `avg candidate_score_calls = 521.45`
  - `avg candidate_score_elapsed_us = 546.50`
  - `avg graph_element_cache_misses = 527.50`
  - `avg graph_neighbor_cache_misses = 48.50`
  - `avg rescan_layer0_seed_elapsed_us = 1495.45`
- ADR-031 disabled:
  - `avg candidate_score_calls = 527.50`
  - `avg candidate_score_elapsed_us = 546.50`
  - `avg graph_element_cache_misses = 527.50`
  - `avg graph_neighbor_cache_misses = 48.50`
  - `avg rescan_layer0_seed_elapsed_us = 1215.70`

Interpretation:

- On this debug seam, ADR-031 on/off barely changes exact-score call count.
- That means another successful ADR-032 slice probably cannot win by merely dropping a few more
  source-local survivors or shuffling frontier bookkeeping.
- The next credible ADR-032 cut needs to change **when** nodes graduate to exact scoring or
  reduce other expensive per-traversal-step work, not just replace tids with slots.

This is still draft-only evidence because the debug SQL surface runs under the `tests` schema and
is not the canonical release benchmark seam. I am using it to choose the next redesign target, not
to supersede the standing warm latency measurements.

## Attempted Third Cut: Exact-On-Head Frontier Promotion

I implemented the first ADR-032 slice that actually changes scan work instead of only moving
state around:

- binary-filtered layer-0 successors are admitted to the frontier with approximate scores
- they are no longer exact-scored immediately during source expansion
- when a frontier candidate reaches the head, the scan promotes it to exact scoring on demand
- if exact scoring makes that candidate worse than the next queued candidate, it is requeued with
  its exact score and the scan keeps going

This keeps the current persisted format and the current ADR-031 binary words, but changes the
exact-score lifecycle from "score survivors eagerly" to "score at frontier head."

## Validation

The code checkpoint is green:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurement: Warm Real-50k Latency

Canonical warm real-`50k`, `m=8`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- `ef=40` run 1: `p50=0.869ms`, `p99=1.559ms`, `mean=0.889ms`
- `ef=40` run 2: `p50=0.875ms`, `p99=1.558ms`, `mean=0.904ms`
- `ef=128`: `p50=1.643ms`, `p99=2.420ms`, `mean=1.657ms`
- `ef=200`: `p50=2.363ms`, `p99=3.509ms`, `mean=2.380ms`

For reference, the kept ADR-031 Tier 1 path was roughly:

- `ef=40`: `mean ~= 1.507-1.510ms`
- `ef=128`: `mean = 3.409ms`
- `ef=200`: `mean = 4.772ms`

So this cut is materially faster across the measured frontier, not just at one point.

## Measurement: Full Real-50k Recall

Full real-`50k`, `1000` queries:

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

## Current Read

This is the first ADR-032 cut that looks like a real keep candidate.

But the result is more nuanced than "strictly better":

- at low `ef_search=40`, the scan is much faster but recall drops materially
- at higher `ef_search`, the latency/recall frontier improves a lot:
  - `ef=128`: `~1.66ms` mean at `0.8861` recall
  - `ef=200`: `~2.38ms` mean at `0.8968` recall

That means the cut is plausibly useful as a better operating frontier, not as a free dominance
upgrade at every `ef_search`.

The important structural result is that this is the first ADR-032 slice where changing the
exact-score lifecycle beats the kept ADR-031 path decisively. That is the strongest evidence so far
that the right ADR-032 lever is algorithmic promotion timing, not node-cache bookkeeping alone.

## Follow-Up Attempt: Promote Before Every Layer-0 Expansion

I then tried the most direct quality-recovery follow-up on top of the kept exact-on-head cut:

- add a promotion-aware layer-0 search helper in `graph.rs`
- exact-score each candidate before it is allowed to expand as a layer-0 source
- requeue the candidate if exact scoring makes it worse than its current approximate rank

This was intended to recover the `ef=40` recall loss by fixing the expansion order itself rather
than only fixing final output ordering.

### Result

This variant is a discard.

The first implementation accidentally removed the original beam-search stop condition and
degenerated into an effectively unbounded layer-0 walk. After fixing that bug, the semantic shape
was still too expensive: the canonical warm real-`50k`, `m=8`, `ef=40` cell no longer completed in
the old millisecond band and spent most of its time inside `initialize_scan_entry_candidate`.

A `perf` sample on the repaired version while the warm `ef=40` cell was running showed:

- `40.61%` `ProdQuantizer::score_ip_from_split_parts`
- `7.98%` `graph::read_page_tuple`
- `5.30%` `cached_graph_element`
- `3.15%` `graph::pop_live_frontier_candidate`
- `2.10%` `graph::push_frontier_and_result_candidate`

Interpretation:

- exact-promoting *every* popped layer-0 source is too expensive
- the helper no longer catastrophically walks the whole graph after the stop-condition fix
- but the promotion scope is still far too wide to keep on the warm `ef=40` seam

The runtime code for this full-promotion follow-up was discarded and the branch was restored to the
last good pushed ADR-032 code state before trying anything else.

## Follow-Up Attempt: Low-Ef Bounded Promotion Budget

I then tried a narrower hybrid follow-up inspired by the reviewer suggestion to make low-`ef`
promotion selective instead of universal:

- keep the existing kept ADR-032 exact-on-head path as the base
- only on low `ef_search` (`<= 64`), allow a tiny layer-0 early-promotion budget
- budget tried here: exact-promote only the first `8` layer-0 expansion candidates

This was meant to recover low-`ef` quality without reintroducing the multi-second blow-up from the
full-promotion variant.

### Measurement: Warm Real-50k Latency

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- `p50=1.051ms`
- `p95=1.491ms`
- `p99=1.741ms`
- `mean=1.080ms`

So the bounded variant returned to a sane latency regime and stayed faster than the kept ADR-031
Tier 1 low-`ef` path.

### Measurement: Full Real-50k Recall

Full real-`50k`, `1000` queries, `m=8`, `ef_search=40`:

- `graph_recall_at_10 = 0.7612`
- `exact_quantized_recall_at_10 = 0.7612`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

Important note: the current `exact_quantized_recall_at_10` comparison on this branch is no longer a
reliable exact-reference field because the reference SQL can itself use the live tqhnsw index. I
am using `graph_recall_at_10` versus fp32 truth as the real quality read here.

### Result

This bounded-promotion variant is also a discard.

Even though latency stayed excellent, the real-`50k` `ef=40` graph recall fell to `0.7612`, which
is worse than the standing kept ADR-032 `ef=40` read of `0.8080`. So a small low-`ef` promotion
budget is not a free middle ground between the kept exact-on-head frontier and the too-expensive
full-promotion layer-0 search.

The runtime code for this bounded follow-up was also discarded and the branch was restored again to
the last good pushed ADR-032 state.

## Updated Read

ADR-032 still looks promising, but the evidence is sharper now:

- changing exact-score timing *at frontier/output consumption* is a real lever
- exact-promoting *every* layer-0 source before expansion is too expensive
- exact-promoting only a tiny low-`ef` source budget is fast, but hurts recall even more

So the next legitimate ADR-032 follow-up should probably not be another "how many sources do we
exact-promote early?" experiment. The better next candidates are:

- a top-`k` frontier lookahead or head-window policy
- score calibration between approximate binary scores and exact scores
- score-budget accounting that promotes only when the scan is materially under-spending exact work

## Follow-Up Attempt: Low-Ef Frontier Head Window

I then tried the most conservative version of the reviewer’s lookahead idea:

- keep the pushed ADR-032 exact-on-head base intact
- only at low `ef_search` (`<= 64`), exact-score a tiny frontier head window before choosing the
  next output/expansion candidate
- window tried here: `4` candidates
- requeue the non-winning window members with their exact scores

This deliberately avoided source-expansion promotion and avoided the earlier multi-second blow-up.

### Measurement: Warm Real-50k Latency

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- `p50=0.794ms`
- `p95=1.101ms`
- `p99=1.334ms`
- `mean=0.814ms`

So the low-`ef` head-window path was extremely fast, even better than the standing kept ADR-032
exact-on-head cut.

### Measurement: Full Real-50k Recall

Full real-`50k`, `1000` queries, `m=8`, `ef_search=40`:

- `graph_recall_at_10 = 0.4507`
- `exact_quantized_recall_at_10 = 0.4507`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

Again, the exact-quantized comparator on this branch is not a reliable exact-reference field, so
the meaningful read is the graph-vs-fp32 `0.4507` recall.

### Result

This head-window variant is a discard.

Despite the outstanding warm latency, the quality collapse is far worse than the standing kept
ADR-032 `ef=40` recall (`0.8080`). That means the windowed exact-choice policy is overfitting hard
to the approximate frontier ordering instead of repairing it.

So the next follow-up should not be another window-size tweak. The remaining plausible seams here
are score calibration or exact-score budget accounting, not more frontier-window heuristics.

## Follow-Up Attempt: Binary-Score Calibration

I then tried the next reviewer-suggested seam: calibrate the binary approximate frontier scores
into the exact-score range before using them for exact-on-head comparisons.

The calibration constants were fit from a real-corpus binary-sign study on the real `50k` corpus:

- `exact_from_binary intercept = 0.013522`
- `exact_from_binary slope = 0.000857`

In scan-score space, that produced:

- `calibrated_scan_score = raw_binary_scan_score * 0.000857 - 0.013522`

The intent was to make the frontier ordering more comparable to exact scores without reintroducing
eager exact work.

### Measurement: Warm Real-50k Latency

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- `p50=0.729ms`
- `p95=1.055ms`
- `p99=1.294ms`
- `mean=0.750ms`

So calibration made the low-`ef` warm seam even faster than the standing kept ADR-032 exact-on-head
cut.

### Measurement: Full Real-50k Recall

Full real-`50k`, `1000` queries, `m=8`, `ef_search=40`:

- `graph_recall_at_10 = 0.6358`
- `exact_quantized_recall_at_10 = 0.6358`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

Again, the exact-quantized comparator on this branch is not a reliable exact-reference field, so
the meaningful read is the graph-vs-fp32 `0.6358` recall.

### Result

This calibration variant is a discard.

It sped the scan up further, but only by making the frontier behave even more like the approximate
binary scorer. That pushed the real `50k`, `ef=40` recall far below the standing kept ADR-032
exact-on-head read (`0.8080`) and confirmed that score-shape tweaks alone are not enough to
recover quality.

The runtime code for this calibration follow-up was also discarded and the branch was restored
again to the last good pushed ADR-032 state.

## Updated Read

ADR-032 still looks promising, but the evidence is sharper now:

- changing exact-score timing *at frontier/output consumption* is a real lever
- exact-promoting *every* layer-0 source before expansion is too expensive
- exact-promoting only a tiny low-`ef` source budget is fast, but hurts recall even more
- low-`ef` frontier head lookahead is fast, but collapses recall badly
- binary-score calibration makes low-`ef` latency even better, but pushes recall much lower

So the next legitimate ADR-032 follow-up should probably not be another "how many sources do we
exact-promote early?" experiment or another score-scale heuristic.

## Follow-Up Attempt: Low-Ef Exact-Score Floor On Best Binary Survivors

I then tried the first score-budget accounting cut:

- keep the pushed ADR-032 exact-on-head base intact
- only at low `ef_search` (`<= 64`), arm a bounded exact-score budget derived from `ef_search`
- budget used here: `min(ef_search / 2, 24)` total exact scores, spent at most `1` per source
  expansion
- spend that budget on the best binary survivor from each expansion before it enters the frontier

This was meant to avoid the earlier "first N source promotions" mistake by spending exact work on
the most promising newly discovered nodes instead of on source nodes themselves.

### Diagnostic: Exact-Score Spend

On a 10-query real-`50k` sample at `m=8`, `ef_search=40`, the score-budget floor changed the hot
path materially:

- `candidate_score_calls` rose from `2.00` to `60.50` on average
- `graph_element_cache_misses` stayed in the same band (`572.80 -> 655.20`)

So this was a real exact-work floor, not a no-op.

### Measurement: Warm Real-50k Latency

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- `p50=0.839ms`
- `p95=1.148ms`
- `p99=1.346ms`
- `mean=0.863ms`

So latency stayed in the same excellent ADR-032 band and remained slightly better than the
standing exact-on-head cut.

### Measurement: Full Real-50k Recall

Full real-`50k`, `1000` queries, `m=8`, `ef_search=40`:

- `graph_recall_at_10 = 0.6774`
- `exact_quantized_recall_at_10 = 0.6774`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

Again, the exact-quantized comparator on this branch is not a reliable exact-reference field, so
the meaningful read is the graph-vs-fp32 `0.6774` recall.

### Result

This low-`ef` exact-score floor is also a discard.

It spent far more exact work than the standing ADR-032 cut, but still recovered much less recall
than expected (`0.6774` vs the standing `0.8080`). That means the missing quality is not solved by
exact-scoring the best candidate from each *local source expansion*. The additional work is being
spent in the wrong place.

## Updated Read

ADR-032 still looks promising, but the evidence is sharper now:

- changing exact-score timing *at frontier/output consumption* is a real lever
- exact-promoting *every* layer-0 source before expansion is too expensive
- exact-promoting only a tiny low-`ef` source budget is fast, but hurts recall even more
- low-`ef` frontier head lookahead is fast, but collapses recall badly
- binary-score calibration makes low-`ef` latency even better, but pushes recall much lower
- a local per-expansion exact-score floor keeps latency excellent, but still fails to recover recall

So the next legitimate ADR-032 follow-up should not be another local-per-source tweak. The better
next candidate is a **global frontier-level exact-work policy**:

- keep a small exact-scored reservoir or budget tied to the live frontier rather than to each
  source expansion
- spend exact work on globally competitive candidates, not on each source's best local survivor

## Success Criteria

- ADR-032 is explicitly documented as a larger scan-runtime redesign, not a cleanup ADR
- the next code slice changes exact-work policy, not just cache/container shape
- the packet is kept live as the current draft while that larger slice is implemented and
  measured
