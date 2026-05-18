# A4 Layer Localization After Hierarchy Fix

## Context

A4 is still blocked after the real hierarchy-collapse fix from review 212.

The corrected deterministic `10k` rebuild changed the live graph picture, but did not clear the
gate:

- corrected hierarchy only, `(m=8, ef=128)`: `27.6%`
- corrected hierarchy + upper-layer carrydown, `(m=8, ef=128)`: `36.4%`
- exact quantized on the same fixed `50` queries: `83.4%`

So the hierarchy bug was real and material, but it was not the last bug.

## Goal

Figure out whether the remaining loss is still:

- bad top-entry selection
- insufficient upper-layer seed width
- build/search objective mismatch
- or a lower graph/runtime problem closer to the layer-1 to layer-0 transition

## New Fixed-Fixture Setup

I stopped using the full gate reset for these localizing probes because it rebuilds both `m=8`
and `m=16`.

Instead, I built a single deterministic `10k / m=8` fixture:

- fixture: `a4layer_10k_m8`
- reset runtime: `4:34.88`
- row count confirmed after later probe interruptions: `10000`

That made the follow-up probes materially cheaper and kept them on the exact same graph.

## What I Measured

### 1. Oracle carrydown by layer

New probe surface:

- `tests.tqhnsw_graph_scan_recall_layer_oracle_k_carrydown_summary_rel(...)`

This probe:

- scores all nodes at or above a chosen layer
- keeps the best `k` seeds for the query
- carries those seeds down through the remaining upper layers
- then runs the normal layer-0 search

Result on fixed `10k`, `m=8`, `ef_search=128`, `50` fixed queries:

- layer `3`, `k=3`: `37.0%`
- layer `2`, `k=3`: `36.8%`
- layer `2`, `k=5`: `37.0%`
- layer `2`, `k=10`: `37.0%`
- layer `1`, `k=3`: `39.0%`
- layer `1`, `k=5`: `39.6%`
- layer `1`, `k=10`: `39.4%`
- layer `1`, `k=20`: `40.2%`
- layer `1`, `k=50`: `41.0%`
- layer `1`, `k=100`: `41.0%`
- exact quantized on the same sample: `83.4%`

## What This Means

These results rule down the earlier “just store more top-level entry points” direction on the
corrected hierarchy.

Why:

- top-level oracle carrydown is flat at the live graph value
- layer-2 oracle carrydown is also flat
- even a very wide layer-1 oracle only reaches `41.0%`

So the remaining miss is not mainly:

- bad single entry-point choice
- or insufficient seed width at the real top layers

The loss is lower than that.

### 2. Build-code exact overlap on the same fixed graph

Existing probe surface:

- `tests.tqhnsw_graph_scan_recall_fixture_summary(...)`

Result on the same fixed `10k / m=8 / ef=128 / 50-query` sample:

- live graph: `37.0%`
- exact quantized: `83.4%`
- build-code proxy: `80.2%`

## Interpretation

This rules down “build/search objective mismatch” as the primary explanation on the current
`4+0` path.

The build objective is not perfect, but it is only `3.2` points below exact quantized on this
sample. The live graph is `46.4` points below exact quantized.

So the dominant remaining issue is not the quantized operating point and not the raw build-code
objective gap. It is in the persisted graph/runtime path.

### 3. Corrected hierarchy summary on the fixed graph

I ported coder-2's per-level hierarchy summary diagnostic into the current tree:

- `tests.tqhnsw_graph_hierarchy_summary(...)`

Result on `a4layer_10k_m8_idx`:

- level `0`: `10000` nodes, neighbor count `16/16/16`
- level `1`: `1249` nodes, neighbor count `8/8/8`
- level `2`: `161` nodes, neighbor count `8/8/8`
- level `3`: `26` nodes, neighbor count `min=3 avg=6.54 max=8`
- level `4`: `3` nodes, neighbor count `3/3/3`

This confirms the corrected persisted hierarchy is real. It is no longer the fake
`10,000`-wide "top layer" from the earlier collapsed build bug, and the lower levels are
densely populated.

## Failed / Invalid Probe To Keep On Record

I added a local-neighborhood coverage probe intended to answer:

- do oracle layer-1 seeds already expose the right layer-0 neighbors directly?

New surfaces:

- `tests.tqhnsw_graph_scan_recall_layer_neighbor_coverage_summary_rel(...)`
- helper script `scripts/pg17_scratch_psql.sh`

Current status:

- the first version was not trustworthy
- its initial `oracle_seed_layer0_neighbor_recall_at_10 = 0` result was caused by a probe bug:
  the helper deduplicated heap tids and then truncated them without preserving score order

I fixed that ordering bug and reran the same probe.

Corrected result on fixed `10k`, `m=8`, `ef_search=128`, `layer=1`, `10` fixed queries, top
`100` oracle seeds:

- live graph recall: `42.0%`
- oracle seed layer-0 neighbor recall: `32.0%`
- exact quantized recall: `83.0%`
- fully covered queries: `2 / 10`
- zero-overlap queries: `3 / 10`

That means the earlier zero result was invalid, but the corrected read is still bad enough to be
useful: even very strong layer-1 seeds do not expose enough good layer-0 candidates directly.

## Current Read

The strongest current read is:

1. hierarchy collapse was real and is fixed
2. upper-layer carrydown helps, but only modestly
3. wider top-level or layer-2 entry sets do not solve the corrected graph
4. even very wide oracle layer-1 seeds only reach `41%`
5. the build-code objective is close to exact quantized on the same sample
6. the corrected layer-1 neighborhood probe only reaches `32%`

That pushes the remaining blocker into the lower persisted graph/runtime path:

- layer-1 to layer-0 transition quality
- serialized lower-layer adjacency quality
- or a remaining runtime/search contradiction over that persisted graph

## Next Step

The next clean split is no longer another SQL oracle variant. It is a direct comparison between:

- the raw `hnsw-rs` graph search on the same deterministic corpus
- and the persisted tqvector graph/runtime on that same corpus

If `hnsw-rs` search is already strong, the remaining bug stays in persistence/runtime. If it is
also weak, the graph construction itself is the dominant ceiling.
