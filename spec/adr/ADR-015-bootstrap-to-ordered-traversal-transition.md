---
id: ADR-015
title: "Bootstrap frontier to ordered traversal transition plan"
status: DECIDED
impact: HIGH for FR-009, FR-006
date: 2026-04-05
---
# ADR-015: Bootstrap frontier to ordered traversal transition plan

## Context

The scan state machine now has two result-production paths operating in sequence:

1. **Bootstrap frontier path**: Consumes scored candidates from a bounded frontier (currently capped
   at `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3`), materializes them into `current_result` + pending
   heap TID drain, and marks them emitted.

2. **Linear scan fallback**: Walks all data pages sequentially, scoring every live element, skipping
   elements already emitted by the frontier path via `emitted_result_tids`.

The `amgettuple` entry point (scan.rs:149-163) tries the frontier path first, then falls through
to the linear scan. This produces correct results (all elements are eventually seen) but not
ordered results (the frontier emits a few good candidates first, then the linear scan fills in
the rest in page order).

The following infrastructure is now in place:

- **Candidate frontier**: `Vec<ScanCandidate>` with score-ordered head selection and consumption
- **Score-ordered expansion**: `BootstrapExpandPolicy::ScoreOrder` picks the best unexpanded
  candidate for adjacency expansion
- **Visited set**: `HashSet<ItemPointer>` prevents re-scoring previously seen elements
- **Expanded-source set**: `HashSet<ItemPointer>` prevents redundant adjacency loads
- **Emitted-result set**: `HashSet<ItemPointer>` prevents duplicate result emission across paths
- **Gamma in element tuples**: Scoring reads gamma from the index, not from heap fetches
- **`score_ip_from_parts`**: Zero-allocation scoring from element tuple parts
- **Direct frontier materialization**: Frontier consumption flows directly to result emission
- **Resolved search-breadth control surface**: the session GUC/reloption precedence model can now
  be implemented independently of planner enablement

## Decision

The transition from bootstrap to ordered traversal SHALL proceed in the following stages:

### Stage 1: Widen frontier to resolved-`ef_search` traversal

Replace `MAX_BOOTSTRAP_FRONTIER_CANDIDATES` with the resolved `ef_search` value (session GUC
override when non-default, otherwise index reloption). Replace the bootstrap
`fill_bootstrap_frontier` / `top_up_bootstrap_frontier` loop with a proper greedy-descent
expansion loop that runs until the frontier is full or all reachable candidates within the search
horizon have been expanded.

The frontier container may remain a `Vec` with `recompute_candidate_frontier_head` until
profiling shows the linear scan matters, or switch to `BinaryHeap` if the code is cleaner.

**Keep the linear scan fallback during this stage.** The fallback provides a safety net while
the frontier is being widened: any element the frontier misses will still be found by the linear
scan. This prevents recall regression during development.

### Stage 2: Result buffering and ordered emission

The current scan emits results one at a time as frontier candidates are consumed. For ordered
scan semantics, the scan must accumulate the top-k results (bounded by `ef_search`) and emit
them in score order. This requires:

- A result buffer (bounded priority queue) that collects scored candidates
- Emission in score order rather than consumption order
- Interaction with the pending-heap-TID drain for duplicate-coalesced elements

The linear scan fallback is still present during this stage but should be exercised less
frequently as the frontier grows to cover more of the index.

### Stage 3: Remove linear scan fallback

Once ordered traversal produces correct, recall-competitive results:

1. Remove the `next_linear_scan_heap_tid` path from `amgettuple`
2. Remove `next_block_number`, `next_offset_number`, and page-iteration state from `TqScanOpaque`
3. The `emitted_result_tids` set can be removed if the frontier's visited set is sufficient
   to prevent duplicate results. If rescan-after-partial-progress requires dedup, it stays.
4. Flip the planner cost gate (ADR-011) to allow the planner to select tqhnsw for ordered scans

### Stage 4: Planner integration

Update `amcostestimate` to return realistic costs based on `ef_search`, estimated graph
fan-out, and index size. This enables the planner to choose tqhnsw when it's the most
efficient path for ORDER BY ... LIMIT queries.

## Resolved questions

### Layer-aware traversal vs. layer-0-only

The current flat neighbor access reads all neighbor TIDs without layer distinction. Full HNSW
greedy descent traverses upper layers (fewer neighbors per node, wider hops) before descending
to layer 0 (denser neighborhood). The question is whether the first ordered traversal
implementation should:

- **Layer-0 only**: Simpler, works for most practical indexes where the entry point is in the
  largest connected component. Misses the logarithmic skip advantage of upper layers.
- **Full multi-layer**: More complex, requires layer-aware neighbor slicing per FR-007's
  `2M` / `M` slot formula. Better asymptotic behavior on large indexes.

The initial ordered traversal SHALL be layer-0-only, with multi-layer greedy descent deferred to a
follow-on optimization when recall or latency data on larger indexes shows it is necessary. The
current merged graph-search scaffolding now follows that boundary: traversal helpers and bootstrap
refill work operate only on layer-0 neighbor access, with no planner or scan code assuming
multi-layer descent yet.

### ef_search default and bounds

ADR-014 budgets traversal memory using the existing `ef_search` default (`40`) and maximum
(`1000`). The remaining work is to wire the resolved control surface into ordered traversal, not to
pick new bounds.

### Scan block count staleness

`scan_block_count` is cached at `amrescan` time and not refreshed. The bootstrap linear scan
uses this to bound page iteration. Once the linear fallback is removed (Stage 3), this field
becomes unnecessary. Until then, it represents a known limitation: concurrent inserts adding
pages after `amrescan` will not be seen by the linear scan pass. This is acceptable under
MVCC semantics.

## Consequences

- The incremental staging allows each step to be validated independently with recall benchmarks
- The linear scan fallback provides a safety net during frontier widening
- The transition can be paused at any stage if recall or performance issues are discovered
- ADR-011's planner cost gate remains in effect until Stage 4
