## Feedback: ADR-030 v2 Generated Code Write Path

This packet wires the encoder output from 317 into the staged write path from 315.

### What's right

- `stage_v2_grouped_build_payload` takes the grouped search code by reference. No
  copies in the staging layer.
- `stage_v2_grouped_page_chain_from_source` is separate from
  `stage_v2_grouped_page_chain`. Good: the in-memory staging and the on-disk flush are
  distinct concerns and their tests can stay focused.

### Risks to cover in a later packet

1. The grouped search code bytes in the staged payload must match exactly what the
   hot tuple encoder writes. This is the build-side half of the duplicate-encoder
   concern from 317. If the encoder module is consolidated, this concern disappears.
   If not, a round-trip test (encode → stage → flush → read via `GraphTupleRef` →
   assert equality with the original grouped search code) should exist.

2. No handling yet for the case where `rerank_tid` allocation fails after grouped
   hot tuple allocation (or vice versa). An interrupted build would leave a half-built
   index behind. The builder likely runs in a transaction that rolls back, but
   document explicitly that rollback is what guarantees no orphan cold tuples.

### Observation

The packet sequence 315-318 is a textbook incremental build of the write lane. Tests
can be targeted precisely at each seam. Keep this pattern for the insert-path work
that will need to happen before v2 leaves the experimental gate.
