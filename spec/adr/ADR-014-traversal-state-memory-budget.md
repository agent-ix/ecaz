---
id: ADR-014
title: "Define traversal state memory budget per scan descriptor"
status: PROPOSED
impact: HIGH for FR-009
date: 2026-04-05
---
# ADR-014: Define traversal state memory budget per scan descriptor

## Context

The upcoming ordered HNSW traversal implementation will add several dynamic data structures to
scan-owned state. Each active scan descriptor will carry:

- **Candidate heap**: A priority queue of scored element TIDs awaiting expansion. Bounded by
  `ef_search` (default 40, max 1000 per FR-009).
- **Visited set**: A set of element TIDs that have already been scored, to prevent re-expansion.
  Grows monotonically during one `amgettuple` sequence. Size depends on graph fan-out and
  `ef_search`.
- **Result buffer**: The top-k scored results ready for emission. Bounded by `ef_search`.

PostgreSQL can run multiple concurrent scan descriptors on the same or different indexes (e.g.,
nested-loop joins, parallel workers, cursor interleaving). Each scan independently allocates its
traversal state. Without explicit budgeting, a high `ef_search` value on a dense graph could lead
to significant per-scan memory consumption, and concurrent scans could amplify this.

The current scan state (`TqScanOpaque`) is a fixed ~200-byte `repr(C)` struct with no dynamic
allocations beyond the prepared query and the copied query payload. The transition to ordered
traversal will introduce the first unbounded-growth data structures in scan-owned memory.

## Decision

Define explicit memory boundaries for traversal state per scan descriptor:

### Candidate heap

- Data structure: `BinaryHeap<ScanCandidate>` or equivalent, ordered by score.
- Maximum capacity: `ef_search` candidates. When the heap is full, a new candidate is only
  inserted if it scores better than the worst current candidate (which is then evicted).
- Memory: `ef_search * size_of::<ScanCandidate>()`. At `ef_search=1000` and ~16 bytes per
  candidate (element TID + score + valid flag + padding), this is ~16 KB. Acceptable.
- Allocation: Rust heap via `Box` or `Vec`, pointer stored in `TqScanOpaque`. Freed in
  `amendscan`. Reset (clear, not reallocate) on `amrescan`.

### Visited set

- Data structure: `HashSet<ItemPointer>` for the initial implementation. If element TIDs are
  dense (sequential block/offset pairs), a bitset indexed by a linearized TID could be more
  compact, but this optimization is deferred.
- Growth bound: In the worst case, the visited set grows to the total number of elements in the
  index (every element reachable from the entry point). For typical HNSW graphs with
  `ef_search=40`, empirical visited-set sizes are 200-500 entries.
- Soft cap: If the visited set exceeds `10 * ef_search` entries, log a warning and stop
  expanding (return whatever results have been collected). This prevents runaway traversal on
  degenerate graphs.
- Memory: At 500 entries and ~16 bytes per `ItemPointer` plus hash overhead, ~16 KB. At the
  soft cap of `10 * 1000 = 10000` entries, ~160 KB. Acceptable for a single scan.
- Allocation: Rust heap, pointer in `TqScanOpaque`. Freed in `amendscan`. Cleared on `amrescan`.

### Result buffer

- Data structure: Bounded priority queue (min-heap by score) of size `ef_search`.
- Same capacity and memory characteristics as the candidate heap.
- Results are popped in order during `amgettuple` calls after the beam search completes.

### Total per-scan budget

At `ef_search=40` (default): ~2 KB candidate heap + ~16 KB visited set + ~2 KB result buffer
= ~20 KB per scan. At `ef_search=1000` (maximum): ~16 KB + ~160 KB + ~16 KB = ~192 KB per scan.

These are well within PostgreSQL's per-backend `work_mem` expectations. No spill-to-disk
mechanism is needed.

### Allocation strategy

All traversal state is allocated on the Rust heap and freed deterministically in `amendscan`.
Raw pointers are stored in `TqScanOpaque` (matching the existing pattern for `prepared_query`
and `query_values`). PostgreSQL memory contexts are not used for traversal state because:

- The data structures are Rust-native (HashMap, BinaryHeap) and don't benefit from palloc.
- Deterministic cleanup via `amendscan` is sufficient; there's no need for memory-context reset
  semantics.
- The scan descriptor's lifetime is well-defined by the executor.

## Consequences

### Benefits

- Explicit bounds prevent unbounded memory growth from high `ef_search` or degenerate graphs.
- The soft cap on visited-set size provides a safety valve without silently corrupting results.
- Per-scan memory is predictable and documented, making capacity planning possible.
- The allocation strategy is consistent with existing scan-owned state patterns.

### Tradeoffs

- The visited-set soft cap means very high `ef_search` values on large, sparse graphs may return
  lower-quality results. This is a deliberate degradation — the alternative is unbounded memory
  or an OOM. The warning log makes the degradation observable.
- `HashSet<ItemPointer>` has higher per-entry overhead than a dense bitset. If profiling shows
  the visited set is a bottleneck, a bitset optimization can be added later without changing the
  API boundary.

## Follow-Up

1. Implement candidate heap and visited set as part of the first ordered-traversal slice.
2. Add `ef_search` GUC parameter (FR-009) to control beam width.
3. Profile visited-set size distribution on real workloads to determine whether the bitset
   optimization is worthwhile.
4. Consider whether parallel scan workers should share or independently allocate traversal state
   (likely independent, matching PostgreSQL's parallel index scan semantics).
