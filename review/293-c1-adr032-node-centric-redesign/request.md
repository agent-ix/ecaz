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

## Success Criteria

- ADR-032 is explicitly documented as a larger scan-runtime redesign, not a cleanup ADR
- the next code slice targets the fused node cache, not another isolated `Arc`/`Vec` swap
- the packet is kept live as the current draft while that larger slice is implemented and
  measured
