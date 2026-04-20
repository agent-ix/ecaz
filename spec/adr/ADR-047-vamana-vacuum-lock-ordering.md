---
id: ADR-047
title: "Vamana Vacuum Graph Repair Lock Ordering (ec_diskann)"
status: ACCEPTED
impact: Affects FR-010 (analog for ec_diskann), ADR-027, ADR-034, ADR-045
date: 2026-04-18
accepted: 2026-04-19
---
# ADR-047: Vamana Vacuum Graph Repair Lock Ordering

## Context

ADR-034 introduces `ecdiskann`, a Vamana (DiskANN) graph access method.
Vacuum against a Vamana index mutates existing neighbor tuples in a
shape superficially similar to `tqhnsw` vacuum, but the underlying
graph structure and the repair rules differ enough that ADR-027's
HNSW vacuum protocol does not apply unmodified.

Key differences from HNSW (ADR-027):

- **Single-layer topology.** The repair pass works on one graph
  level. There is no per-layer repair ordering; the only ordering is
  physical page ordering.
- **Fixed-`R` neighbor lists instead of per-layer slices.** A
  `ecdiskann` node's neighbor tuple is a single fixed-capacity list
  of size `R`. Repair never has to repartition a multi-slice tuple;
  it only has to replace dead TIDs with live ones within that flat
  list.
- **α-pruning bounds replacement selection.** ADR-027's `tqhnsw`
  repair pass currently fills only `INVALID` slots with candidates
  discovered through read-only replanning. The Vamana analogue keeps
  that fill-only posture but scores candidates through `RobustPrune`
  so that the rebuilt list preserves the α-dominance property that
  gives Vamana its diversity guarantee.
- **Entry-point medoid is not vacuumed in place.** If the entry-
  point TID itself is dead after pass 1, repair does not migrate the
  medoid during pass 2. Instead, the metadata page records that the
  medoid is stale, and a follow-up rebuild or the next scheduled
  vacuum reruns the medoid-approximation step. Vacuum never
  reacquires the metadata page under an exclusive lock concurrent
  with a data-page exclusive lock.
- **Cold rerank-payload pages follow the same ordering.** PqFastScan
  hot/cold split means a Vamana node occupies a hot page plus a
  cold rerank page (for fresh-payload indexes). Vacuum must honor
  ordered lock acquisition across both chains.

The rest of the invariants from ADR-027 still apply: build delete-set
first, scan in ascending block order, one data-page `EXCLUSIVE` lock
at a time, no metadata overlap with data-page exclusive lock, replan
read-only between ordered write passes, fill-only writes.

## Decision

Vamana vacuum graph repair follows this write order:

1. **Pass 1 — dead-heap-TID discovery.**
   Scan every data page (hot chain plus cold chain) under
   `BUFFER_LOCK_SHARE`. For each element tuple whose heap TID is in
   the vacuum delete-set, record the element TID in the
   delete-set. Strip the dead heap TIDs under a narrow per-page
   `BUFFER_LOCK_EXCLUSIVE`. No neighbor mutation in pass 1. Cold
   rerank payload pages are scanned and cleaned the same way.

2. **Pass 2 — neighbor-array repair.**
   Scan data pages in ascending block order under
   `BUFFER_LOCK_SHARE` and decide page-by-page whether repair is
   needed. If a neighbor tuple references any element in the
   delete-set, that page needs repair.

3. **Repair planning is read-only.**
   Before taking a data-page `EXCLUSIVE` lock, compute the
   replacement candidate list for every dead neighbor on that page.
   Candidate generation runs the same greedy search helper scan
   uses, starting from the live medoid entry point. The read-only
   plan produces a sorted candidate list per target slot.

4. **Write phase: reopen one page under `BUFFER_LOCK_EXCLUSIVE`.**
   Release the share lock before taking the exclusive lock. Never
   upgrade in place. That one page-local write window may update
   multiple neighbor tuples that live on the same page; the
   ordering rule is per physical page, not per neighbor tuple.

5. **Fill-only writes under the exclusive lock.**
   Dead neighbor TIDs become `INVALID` (unlink repair). Only
   currently free slots and slots being unlinked as dead are filled
   from the read-only plan. Live neighbors are never evicted under
   the page write lock, even if a planned candidate would dominate
   them under α-pruning. Live-neighbor eviction at vacuum time is
   explicitly out of scope; any residual quality gap is addressed
   at rebuild rather than under a page exclusive lock.

6. **One data-page `EXCLUSIVE` lock at a time, in ascending block
   order.**
   Identical rule to ADR-027 step 4. The per-page write is one
   `GenericXLog` transaction. When the write page and its cold
   rerank payload chain both require updates (e.g., the cold
   payload needs a heap-TID strip too), the cold page lock is
   acquired after the hot page lock for that element and released
   before moving to the next hot page in block order.

7. **No metadata-page `EXCLUSIVE` lock during pass 2.**
   Identical rule to ADR-027 step 5.

8. **Replan-before-retry across ordered passes.**
   If a read-only plan discovers stale live-neighbor state when
   reopened under the exclusive lock (for example, a concurrent
   insert already repaired a slot), finish the current ordered
   page pass without fabricating a replacement, then rerun
   read-only planning for the affected targets before the next
   ordered pass begins. Same retry shape as ADR-027 step 6 and
   ADR-026 step 7, and identical to ADR-046 step 7 at the insert
   side.

9. **Pass 3 — finalization.**
   After pass 2 completes, scan again under `BUFFER_LOCK_SHARE` to
   identify elements that are now orphans (all heap TIDs dead, no
   live inbound neighbor references). Mark `deleted = true` under a
   per-page `BUFFER_LOCK_EXCLUSIVE` `GenericXLog` transaction. Pass
   3 does not traverse the graph or run `RobustPrune`; it is a
   page-local flip only.

10. **Entry-point medoid repair is deferred.**
    If the entry-point TID is in the delete-set after pass 1, pass
    2 rebuilds inbound neighbor arrays to no longer reference it,
    but does not migrate the entry point itself. A metadata
    `needs_medoid_refresh` flag is set under a narrow metadata-
    page exclusive lock (taken after all data-page writes in the
    current vacuum run have committed). The next rebuild or the
    next scheduled maintenance pass reruns medoid approximation.

In short: ordered page scan, read-only replan, one data-page write
lock at a time, fill-only writes, no metadata overlap, medoid
migration deferred to rebuild.

### Frozen implementation rules (2026-04-19 review)

These seven rules freeze the implementation answers so Phase 8B can
proceed without re-opening the ADR. Where they differ from the
original step list above, the frozen rule wins.

1. **Pass 1 is one interleaved ordered scan, not two global passes.**
   For each page in ascending block order: open under
   `BUFFER_LOCK_SHARE`, discover dead primary heap TIDs on that page,
   and — if any need stripping — release `SHARE`, reopen the same
   page alone under `BUFFER_LOCK_EXCLUSIVE`, strip them, WAL-log
   once, release. Then continue to the next page. Matches the Phase
   8A primitive boundary and keeps the pass count honest: pass 1,
   pass 2, pass 3.

2. **Medoid fallback: first live tuple.** If the metadata
   entry-point tuple is in the delete-set or already tombstoned,
   read-only planning falls back to the **lowest-block, lowest-offset
   live tuple** in the chain (the same `resolve_entry_point →
   first_live_tid` rule the scan path uses). Planning never runs
   from a dead medoid.

3. **Pass 3 finalizes on fully-dead tuples, not inbound-orphan
   analysis.** A tuple is finalized (i.e., `deleted = true` flipped)
   when it is fully dead: `primary_heaptid == INVALID &&
   !has_overflow_heaptids`. Tombstoned tuples may still retain
   neighbor payload and may still be traversed until a later vacuum
   reaps them; that matches the Phase 8A `mark_deleted` contract and
   avoids an O(N·R) inbound-edge bookkeeping surface.

4. **V0 vacuum writes no cold rerank chain.** Pass 3 flips the hot
   tuple's `deleted` bit only. There is no index-side cold payload
   to sweep, because V0 insert never wrote one (ADR-046 frozen rule
   1). A future ADR-044 C1 reopen is the only path that adds a
   pass-3 cold-cleanup rule.

5. **Repair replan cap.** `MAX_REPAIR_REPLAN_PASSES = 3` total
   ordered repair passes per vacuum run. On exceed, log loudly and
   leave remaining dead / `INVALID` slots for a later vacuum rather
   than spinning. Reuses the insert-side precedent from ADR-046.

6. **Vacuum and live insert run concurrently.** Both use the same
   ordered-page-pass discipline and resolve write/write races
   through bounded read-only replanning rather than global
   serialization. No generation counter is required; stale
   detection compares reopened tuple contents against the
   read-only plan.

7. **`needs_medoid_refresh` ownership is monotonic and
   vacuum-only.** Vacuum sets `needs_medoid_refresh = true` when
   the current entry-point tuple becomes fully dead / tombstoned.
   Insert does **not** set this flag (ADR-046 frozen rule 5). Only
   rebuild (or a future explicit medoid-refresh maintenance path)
   clears it. This prevents ADR-046 and ADR-047 from fighting over
   the same metadata decision bit.

### Implementation corollaries

- `RobustPrune` stays in the read-only planning window. Vacuum
  repair is **best-effort connectivity repair**: planning may
  evaluate `RobustPrune` over the union of live neighbors plus
  proposed candidates, but the write phase only fills slots that
  are dead or `INVALID` with the accepted candidates. Live
  neighbors are never evicted under the page lock. If the accepted
  set is smaller than the free-slot count, remaining slots stay
  `INVALID`. This is the correct conservative contract for a
  concurrent vacuum.
- Insert's duplicate-binding lookup (ADR-046) must ignore tuples
  that are tombstoned OR have `primary_heaptid == INVALID`. Once
  pass 1 strips the heap TID, the node is no longer eligible for
  duplicate binding, so pass 3 is monotonic with respect to
  concurrent insert.

## Consequences

### Positive

- Pass 2 repair can touch arbitrary neighbor pages without
  introducing mixed lock order, matching the safety property
  ADR-027 gives `tqhnsw`.
- α-aware candidate selection preserves Vamana's diversity
  invariant across repair without widening the deadlock surface —
  α-pruning runs over the target's existing neighbor list and the
  read-only candidate list, both of which are materialized outside
  the page write window.
- The fill-only posture means repair never regresses live
  connectivity; the worst-case outcome of a race is leaving an
  `INVALID` slot for the next vacuum to fill, not deleting a live
  edge.
- Hot/cold page chains share one ordered write protocol. No
  separate cold-vacuum pass is required.

### Negative

- Fill-only writes do not enforce α-dominance globally. A slot
  filled at vacuum time may be dominated under α by a live
  neighbor that was never evicted. Result: slight recall drift
  across a long vacuum history, bounded by the total delete rate
  between rebuilds.
- Medoid migration is deferred to rebuild, so an index that
  deletes the current entry point loses a near-optimal starting
  seed until the next rebuild or scheduled medoid refresh. Scan
  still terminates correctly but may explore more candidates
  before converging.
- Read-only replan can happen more than once per vacuum run under
  write-heavy contention. Bounded in practice; unbounded in
  theory if the retry always loses the race. Same risk-bounding
  posture as ADR-027.

### Neutral

- This ADR constrains lock acquisition order only. It does not
  change tuple layout or WAL format.
- Finalization stays a separate page-local pass after graph
  repair, matching ADR-027.
- Post-vacuum recall target (≥ 80% of pre-vacuum after 10%
  deletion) is the same bar `tqhnsw` hits under FR-010-AC-2.

## References

- ADR-027: Vacuum Graph Repair Lock Ordering (HNSW)
- ADR-026: Live Insert Backlink Lock Ordering (HNSW)
- ADR-034: DiskANN as Second Access Method
- ADR-046: Vamana Live Insert Lock Ordering (ecdiskann)
- pgvectorscale, `pgvectorscale/src/access_method/vacuum.rs` —
  reference implementation. pgvectorscale's current vacuum is
  simpler than this protocol because it does not support live
  insert under the same ADR-046-style α-aware repair; that is why
  this ADR does not port its rules directly.
- Subramanya et al., *DiskANN: Fast Accurate Billion-Point Nearest
  Neighbor Search on a Single Node*, NeurIPS 2019. `RobustPrune`
  §3.2 — the function reused under step 5 at fill time.
