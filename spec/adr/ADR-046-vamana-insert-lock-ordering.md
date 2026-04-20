---
id: ADR-046
title: "Vamana Live Insert Lock Ordering (ec_diskann)"
status: ACCEPTED
impact: Affects FR-016 (analog for ec_diskann), ADR-026, ADR-034, ADR-045
date: 2026-04-18
accepted: 2026-04-19
---
# ADR-046: Vamana Live Insert Lock Ordering

## Context

ADR-034 introduces `ecdiskann`, a Vamana (DiskANN) graph access method.
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

### Frozen implementation rules (2026-04-19 review)

These six rules freeze the implementation answers so Phase 7 can
proceed without re-opening the ADR:

1. **V0 insert writes only hot graph pages plus metadata.** There is
   no index-side cold rerank payload chain. `rerank_tid` remains
   `INVALID` and `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` is not set by
   insert. Exact rerank is served from the heap `ecvector` row per
   ADR-044. A future ADR-044 C1 reopen is the only path that adds
   insert-time cold writes.

2. **Overflow heaptid chain growth follows the same ordered rule.**
   When an incoming row is a duplicate-vector bind to an existing
   node:
   - If the node absorbs the new heap TID in place, it is an ordinary
     step-5 ordered rewrite of that tuple's page.
   - If the chain must grow, step 2 appends the fresh overflow tuple
     under one isolated page `EXCLUSIVE` lock and releases it, then
     step 5 patches the head / predecessor tuple on a later ordered
     page pass. Append-like allocation first, ordered rewrites second,
     metadata last.

3. **Write-window scoring is page-local.** `RobustPrune` under the
   target page's `EXCLUSIVE` lock consumes only inputs materialized
   during the read-only planning pass or carried inline on the target
   tuple itself (ADR-045 Decision 3). Fetching remote candidate
   payloads while holding a data-page `EXCLUSIVE` lock is forbidden.

4. **Stale-target retry cap.** Reuse the HNSW insert-path name:
   `MAX_BACKLINK_REPLAN_PASSES = 3` total ordered write passes per
   insert. On exceed, log loudly and abandon remaining targets for
   a later insert. Do not invent a Vamana-specific cap until
   measurements demand one.

5. **Insert does not own `needs_medoid_refresh`.** Live insert owns
   exactly: append or duplicate-bind, backlink repair, and
   `inserted_since_rebuild += 1` for true new-node inserts. The
   `needs_medoid_refresh` flag is monotonic and written only by
   vacuum (ADR-047) or a future explicit medoid-refresh maintenance
   path. This prevents two independent writers racing on the same
   metadata decision bit.

6. **Insert and vacuum run concurrently.** `ec_diskann` insert and
   vacuum do not serialize through a global AM lock. They tolerate
   each other's page-local drift through bounded stale-target retry
   between ordered write passes. Stale detection compares the
   reopened tuple contents against the read-only plan; no generation
   counter is required.

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
  ADR-026, so the `tqhnsw` and `ecdiskann` insert paths can share
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
  constrains lock acquisition order on the `ecdiskann` insert path.
- Medoid drift is out of scope for this ADR. The entry-point medoid
  is recomputed only at rebuild time in v0. A future live-medoid
  migration ADR will extend this protocol if it is ever adopted.

## References

- ADR-026: Live Insert Backlink Lock Ordering (HNSW)
- ADR-034: DiskANN as Second Access Method
- ADR-047: Vamana Vacuum Graph Repair Lock Ordering (ecdiskann)
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
