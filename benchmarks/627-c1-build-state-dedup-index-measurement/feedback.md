# Feedback: 627 BuildState Dedup Index Measurement

## Verdict: Accept

`HashMap<BuildTupleDedupKey, usize>` replacement is correct. The measurement
closes the O(N²) bottleneck identified in packet 626.

## Code Review

**`BuildTupleDedupKey { gamma_bits: u32, code: Vec<u8> }`**: Using
`gamma.to_bits()` as the equality field rather than float comparison is
correct — `f32::eq` has NaN hazards; bit-level equality is the right key type
for a hash map. The code is already `Vec<u8>` so cloning into the key is
straightforward.

**`derive(Hash, Eq, PartialEq)` on key**: Correct. `Vec<u8>` implements these.
`u32` implements them. The derived implementations are exact-byte equality,
which is what duplicate coalescing requires.

**Index maintenance**: `tuple_index_by_payload.insert(dedup_key, tuple_idx)` is
called only for new (non-duplicate) tuples. Existing duplicate entries merge
their heap TIDs without inserting a new key. The index remains consistent as
long as the existing tuple's index in `heap_tuples` does not change — and it
cannot, because tuples are only appended, never reordered during the push loop.

**Test `build_state_push_indexes_payloads_for_duplicate_coalescing`**: Covers
the same-code, different-heap-tid case (two tuples merged) and the
different-code case (two separate entries). This is the correct test contract
for the dedup index.

**`insert.rs` change**: Minor — updates a reference that needed updating after
`BuildState` added the new field. Not an independent concern.

## Result

- Serial sort/push: 1,471 ms (was 8,618 ms heap ingest for serial path — this
  is a different counter but the improvement is consistent).
- Parallel sort/push: 157 ms (was 6,706 ms).
- Serial graph unchanged: ~47,023 ms. Parallel graph unchanged: ~45,105 ms.

The O(N²) duplicate scan was the bottleneck at 50k. It is now O(N) (one map
lookup per tuple). The 97.7% reduction in sort/push time is the expected result.

## No Issues
