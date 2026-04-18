## Feedback: PqFastScan Vacuum On Built Indexes

Read `count_element_tuples` dispatch in `src/am/shared.rs`, the pass-1
updates `ElementVacuumUpdate::{TurboQuant, PqFastScanHot}` in
`vacuum.rs:216-610`, `repair_graph_connections_with_storage` at
`:616`, and `finalize_fully_dead_elements_with_storage` at `:1631`.

### What's right

- **Same three-phase vacuum lifecycle, just storage-aware.** Packet
  framing — "vacuum does not need a separate algorithm, it needs
  storage-aware tuple decode/rewrite" — is the right insight. Pass
  1 strips dead heap TIDs, pass 2 repairs broken edges, pass 3
  tombstones fully-dead tuples. All three now branch on storage
  descriptor instead of assuming scalar.
- **Pass 1 rewrites only the inline heap-TID list for grouped
  hot tuples.** The rest of the grouped hot payload (binary sidecar,
  search code, reranktid pointer) is preserved in place. That
  matches the build-time layout and keeps vacuum from re-encoding
  payloads it doesn't need to touch.
- **Repair-request discovery reads `(level, deleted,
  heaptids_empty, neighbortid)` from either tuple shape.** That
  four-tuple is the shared shape vacuum actually cares about for
  deciding "this edge is broken" — lifting it out of the concrete
  tuple type is the right factoring.
- **Finalization only tombstones after pass 1 clears the last
  heap TID.** Same ordering invariant as scalar. Worth a comment,
  but correct.

### Concerns

1. **No grouped linear top-up yet in this packet.** The packet
   ships pass 1/2/3 parity but leaves `LinearRepairPlanner` scalar-
   only. That means grouped repair can under-fill broken neighbor
   slots under certain graph-search-starved conditions. 385 closes
   this, but the combination (383 without 385) is a half-done
   repair story — grouped vacuum is *safer* than no vacuum but not
   recall-equivalent to scalar vacuum. Worth noting explicitly in
   the task-15 landing bar that 383+385 ship together.

2. **Rerank tuple lifecycle under vacuum.** Pass 1 rewrites the hot
   tuple's inline heap TID list but does not touch the rerank
   tuple. That's correct because the rerank payload is content-
   addressed to the logical node, not the heap TIDs. But if a
   future change ever made rerank tuples point-dependent on
   heap-TID identity, this separation would silently break.
   Asserting the invariant ("rerank payload is heap-TID-
   independent") somewhere, either in page-layout docs or as a
   `debug_assert`, would protect it.

3. **Duplicate-heap-TID pass-1 compaction test.** The new pg
   coverage exercises duplicate compaction — that's the right
   thing given the duplicate insert race from packet 382. Worth
   confirming the test actually creates the racing-insert shape
   (two hot tuples for the same code), not just a duplicate-HeapTID-
   within-one-tuple shape. The packet description doesn't fully
   disambiguate.

4. **Linker gap.** Three pg tests are the load-bearing proof for
   this packet: grouped stats, grouped duplicate compaction,
   grouped dead-edge unlink + finalize. None ran locally. The
   repair-search and finalization code paths are the ones that
   produce broken-graph corruption if wrong, and they need real
   pgrx test coverage before merge.

### Observation

This plus 385 is the vacuum parity story. Read together they match
what `TurboQuant` vacuum has had for months. Read alone, 383 is
under-complete.
