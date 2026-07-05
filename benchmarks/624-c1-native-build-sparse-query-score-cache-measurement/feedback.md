# Feedback: 624 Native Build Sparse Query Score Cache Measurement

## Verdict: Accept

Generation-stamped cache is correct and the modest improvement justifies the
change. Neutral parallel result is correctly interpreted.

## Code Review

**`NativeBuildQueryScoreCache`**: The generation-stamp pattern (wrapping u32
counter, fill-to-zero on wrap) is correct. The wrap case clears the
`generations` vec and resets to 1, so generation 0 is never a valid current
generation. This prevents stale reads after wrap. Identical in structure to the
`NativeBuildVisitedSet` introduced later in packet 628.

**`wrapping_add(1)` on `current_generation`**: Correct. Overflow is handled
explicitly before the generation value is used.

**One cache per build vs one per node**: The change eliminates
`vec![None; heap_tuples.len()]` allocation for every inserted node. At 10k
rows with m=6, that was ~10k allocations of 10k-element vecs. The cache is
now allocated once and reset by advancing the generation. This is the right
structure.

**Score storage type `Vec<Option<f32>>`** (before) → **`Vec<u32>` generations
+ `Vec<f32>` scores** (implied by the generation/cache pattern): The new
approach stores a generation stamp per slot and the score separately. Correct
and allocation-free per node.

## Result

- Serial graph phase: 8197 ms → baseline, vs 8384 ms in packet 622. Modest
  improvement (~2.2%) on serial.
- Parallel graph phase: essentially flat (8220 ms vs 8214 ms in 622).
- The change removes a real O(N) per-node allocation; that it produces a modest
  result on 10k is expected — the allocation pressure is real but not the
  dominant cost.

## Conclusion

The evidence supports continuing to optimize graph construction rather than
returning to parallel heap ingest tuning. The parallel path is neutral on this
fixture because graph assembly is still serial. Correct direction.

## No Issues
