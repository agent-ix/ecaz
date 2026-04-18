---
id: ADR-041
title: "Vamana Live Insert Lock Ordering (tqdiskann)"
status: PROPOSED
impact: Affects FR-016 (analog for tqdiskann), ADR-026, ADR-034
date: 2026-04-18
---
# ADR-041: Vamana Live Insert Lock Ordering

## Context

ADR-034 introduces `tqdiskann`, a Vamana (DiskANN) graph access method.
Live insert against a Vamana graph mutates existing neighbor tuples in a
shape superficially similar to `tqhnsw` live insert, but the underlying
graph structure and the mutation rules differ enough that ADR-026's
HNSW lock-ordering protocol does not apply unmodified.

Key differences from HNSW (ADR-026):

- **Single-layer topology.** There is no upper-layer dance. All
  mutations happen on one graph, so there is no layered lock ordering
  per layer.
- **α-pruning replaces score-only top-M eviction.** When a target
  node's neighbor list is full, Vamana's `RobustPrune` routine
  rebuilds the list from the union of the existing neighbors plus
  the proposed new neighbor, using an α-relaxed Euclidean filter to
  prune dominated candidates. This is strictly more work under the
  page write lock than HNSW's current score-only top-M rule.
- **Entry point drift.** There is no level-based entry-point
  promotion rule. The entry point is the persistent medoid, and live
  insert does not move it. Metadata mutations during insert are
  therefore narrower than HNSW's: only counters and optional
  drift-trigger fields, not a layer-specific entry payload.
- **No upper-layer backlinks to serialize against.** Every backlink
  is at the same graph level, which simplifies the ordering rule.

The rest of the invariants from ADR-026 still apply, because they come
from Postgres buffer-lock semantics rather than HNSW specifics:
deterministic ascending-page lock order, single-page exclusive write
lock at a time, no metadata overlap with data-page exclusive locks,
and no replanning inside a held data-page exclusive lock.

## Decision

Vamana live insert follows this write order:

1. **Traverse read-only first.**
   Greedy-search from the medoid entry point to discover the
   insert-time candidate list of size `L` uses only
   `BUFFER_LOCK_SHARE` on data pages. No neighbor tuple is rewritten
   during traversal. The candidate list, scored code, and
   `RobustPrune`-selected neighbor set are all computed before any
   exclusive lock is taken.

2. **Append the new node under one data-page `EXCLUSIVE` buffer lock.**
   The append path may allocate a fresh page, but it holds only that
   single data-page write lock. The node tuple carries its final
   selected forward neighbor list inline — Vamana's single-layer
   topology means the forward-link set is complete at append time.

3. **Release the append page lock before any backlink work starts.**
   Identical rule to ADR-026 step 3. Eliminates deadlock risk
   between append and backlink phases.

4. **Collect backlink targets and sort by physical
   `(block_number, offset_number)`.**
   The backlink target set is exactly the forward-neighbor set the
   new node ships with. For each target, the backlink operation may
   either append the new node's TID to a target whose neighbor list
   is not full, or run `RobustPrune` if it is full. Both outcomes
   must be planned read-only before the write phase begins.

5. **Rewrite existing neighbor tuples one data page at a time in
   ascending block order.**
   When multiple target tuples live on the same page, update them
   under one `BUFFER_LOCK_EXCLUSIVE` and one `GenericXLog`
   transaction. Within a page, update in ascending offset order.
   This invariant is identical to ADR-026 step 5.

6. **α-pruning inside the page write window is bounded.**
   `RobustPrune` is a pure function of the target's current neighbor
   list, the new candidate, persisted PqFastScan codes, and the
   configured α. All inputs are either already in the target's on-
   page tuple or came in from the read-only planning pass. The
   prune itself does not acquire additional buffer locks and does
   not read other data pages. If the prune discovers that the
   target's current neighbor list has drifted since the planning
   pass (e.g., a concurrent inserter already added the new node's
   TID, or pruned a candidate we expected to be present), the write
   is abandoned for that target and added to the stale-target
   retry list. The page write lock is released before any retry.

7. **Stale-target retry re-enters read-only planning.**
   When a target is marked stale under the page lock, record only
   `(target_element_tid)` plus the already-selected new-node
   backlink payload. No page-derived or lock-protected data escapes
   the write phase into the retry phase. After the current ordered
   page pass completes, restart read-only planning for those
   targets and make another ordered write pass. Same escape valve
   as ADR-026 step 7, adapted to single-layer Vamana semantics.

8. **Acquire the metadata-page `EXCLUSIVE` lock only after all data-
   page writes are complete.**
   Metadata mutations during Vamana insert are narrower than HNSW's:
   `inserted_since_rebuild` increment, optional entry-point-medoid
   drift counter, format-version invariants. No entry-point TID
   mutation during live insert (the medoid does not migrate).
   Metadata promotion never overlaps a data-page `EXCLUSIVE` lock.

9. **First-insert bootstrap is serialized under the metadata page.**
   Empty-index first insert initializes metadata and plants the
   first node (which becomes the initial entry-point medoid) under
   one metadata-page `EXCLUSIVE` lock. Identical shape to the
   `tqhnsw` bootstrap path. The medoid-approximation pass for a
   bulk build does not run at live-insert time.

In short: traverse read-only, append under one data-page lock,
release, sort backlink targets by physical page, run α-prune per
page under one page exclusive lock each, retry stale targets
through a fresh read-only plan, metadata last.

## Consequences

### Positive

- Concurrent live inserts share a deterministic data-page lock order,
  matching the safety property ADR-026 gives `tqhnsw`.
- α-pruning runs inside the narrow page-local write window, so its
  cost is bounded by the target's neighbor-list size `R` rather than
  by graph-wide state. For `R = 32` this is a few-dozen score
  evaluations against per-node PqFastScan codes already cached on
  the page.
- No layered lock ordering to get wrong. Single-layer Vamana
  simplifies the protocol relative to HNSW.
- The stale-target retry loop reuses the same control-flow shape as
  ADR-026, so the `tqhnsw` and `tqdiskann` insert paths can share
  the ordered-page-pass harness even though the per-target work
  differs.

### Negative

- α-pruning is per-target work under the page lock. Backlink install
  is strictly more expensive than HNSW's top-M top-up on a full
  slice, especially when many targets overflow simultaneously.
- The stale-target retry may need multiple read-only planning passes
  under write-heavy contention. Bounded in practice by the number
  of concurrent inserters touching the same target, but
  theoretically unbounded if the retry always loses the race.
  Mitigation: cap retries per insert with a loud warning on exceed,
  same posture ADR-026 takes for full-slice retries.
- No entry-point-level short-circuit on insert. Every insert walks
  from the medoid, which is more read work per insert than HNSW's
  level-aware descent at similar corpus size. This is an intrinsic
  property of single-layer Vamana rather than a choice of this ADR.

### Neutral

- First-insert bootstrap remains on the existing metadata-lock path
  because it does not yet touch multiple data pages. Same rule as
  ADR-026.
- This ADR does not change tuple layout or WAL format; it only
  constrains lock acquisition order on the `tqdiskann` insert path.
- Medoid drift is out of scope for this ADR. The entry-point medoid
  is recomputed only at rebuild time in v0. A future live-medoid
  migration ADR will extend this protocol if it is ever adopted.

## References

- ADR-026: Live Insert Backlink Lock Ordering (HNSW)
- ADR-034: DiskANN as Second Access Method
- ADR-042: Vamana Vacuum Graph Repair Lock Ordering (tqdiskann)
- pgvectorscale, `pgvectorscale/src/access_method/insert.rs` — reference
  implementation of Vamana live insert with α-pruning under Postgres
  page locks. Known-rough concurrency story per ADR-034 context; this
  ADR intentionally diverges on the read-only-replan posture.
- Subramanya et al., *DiskANN: Fast Accurate Billion-Point Nearest
  Neighbor Search on a Single Node*, NeurIPS 2019. `RobustPrune`
  definition §3.2.
- Singh et al., *FreshDiskANN: A Fast and Accurate Graph-Based ANN
  Index for Streaming Similarity Search*, 2021. Streaming-insert
  variant that informs the stale-target retry rule above.
