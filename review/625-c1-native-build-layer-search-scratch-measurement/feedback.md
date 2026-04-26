# Feedback: 625 Native Build Layer Search Scratch Reuse

## Verdict: Accept

Scratch reuse is a material win. The local duplication is the right tradeoff.

## Code Review

**`NativeBuildLayerSearchScratch`**: Owns `visited: NativeBuildVisitedSet` (see
packet 628 for the generation-stamp version; at this point it's a `HashSet`),
`candidate_points: BinaryHeap<Reverse<...>>`, and
`result_points: BinaryHeap<...>`. All three are cleared via `scratch.clear()`
rather than dropped and reallocated per node. This is the correct structure.

**Local duplication of layer search loop**: The concern in the review focus is
whether duplicating the generic layer search locally is acceptable. It is:
- The native build search operates on `BuildState` indices, not on page/scan
  graph structures. A shared generic helper would need to abstract over two
  incompatible graph representations.
- The duplication is isolated to `build.rs`. Scan/vacuum behavior is
  unchanged.
- If the build graph representation converges with the scan graph later, the
  deduplication can happen at that point.

**Ordering and tie-break semantics**: The `NativeBuildLayerSearchCandidate`
orders by `total_cmp` on score, matching `BeamCandidate` semantics. The
tie-break by `sequence` (insertion order) prevents non-determinism when
scores collide. This is consistent with the generic beam search.

**`result_points: BinaryHeap`**: The result heap is the ef-bounded collection.
The pruning condition (`result_points.len() >= ef_search && candidate.score
> worst_result`) matches the standard HNSW beam search cutoff. Correct.

## Result

- Serial graph phase: 8197 ms (624) → 7304 ms (~10.9% improvement).
- Parallel graph phase: 8220 ms (624) → 7184 ms (~12.6% improvement).
- Total build wall time: serial ~7925 ms, parallel ~7665 ms. Parallel is ~3.3%
  faster — first time parallel beats serial on this fixture.

This is a material improvement across both paths. The scratch allocation
elimination is clearly load-bearing at 10k × 64 × m=6.

## No Issues
