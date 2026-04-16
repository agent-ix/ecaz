## Feedback: ADR-030 v2 Persisted Grouped Codebook Basis

A batched packet. Substantially larger than the last dozen seam packets — five
tightly coupled changes (query LUT builder, persistence tuple, metadata pointer,
builder persist step, scan-side load helper). The packet is honest about being
batched; it also says *why* batching is appropriate here ("closely related
prerequisites"). That's the right call — these five items have no useful
intermediate checkpoint where any one of them is individually testable end-to-end.

### What's right

- Codebook persistence uses the existing DataPageChain shape (`TqGroupedCodebookTuple`
  + insert/read/update helpers + head pointer in metadata). Not a parallel page
  format. That keeps vacuum, FSM, and WAL replay boring.
- `GraphStorageDescriptor::from_metadata` was tightened to reject grouped-v2
  without a persisted codebook chain. One more field in the metadata contract
  means one more thing the runtime cannot silently guess.
- `build_grouped_pq_lut_f32` in `src/quant/grouped_pq.rs` uses the same flat
  codebook layout that both study harness and runtime will read. Study/runtime
  LUT building cannot drift from here without a compile break.
- End-to-end pg-test (`test_grouped_v2_graph_reads_load_persisted_codebooks`):
  build writes codebooks to disk, reader loads them back, layout round-trips.

### Concerns

1. **Batching discipline.** This was the right time to batch, but batching more
   than five interlocked items in future packets risks making review bisection
   harder. If a single future packet must ship: write path, read path, scan
   integration, *and* a score-math change together, the review payload becomes
   unwieldy. Keep the bar high for batching — "strictly prerequisite for the next
   real runtime step" rather than "convenient to land together."

2. **Codebook storage size.** 96 groups × 16 centroids × 16 dims × f32 = 96 KB per
   index for 1536-dim. That fits on a few DataPages. For very small indexes the
   codebook may dominate index size. Worth recording expected codebook size in
   the ADR so operators can reason about storage overhead of v2.

3. **`bench_api` surface (again).** `build_grouped_pq_lut_f32` is now exported
   through `bench_api`. That export surface is growing (packer, scorer, now LUT
   builder). Each export is a load-bearing layout contract. Worth a single
   export contract comment somewhere in `bench_api` that names which layout
   invariants any external caller depends on.

4. **Metadata rejection coverage.** The new
   `graph_storage_descriptor_rejects_grouped_v2_missing_codebook_head` test is
   good. But now the metadata validation has seven distinct reject branches
   (from packet 341 + codebook head). A parametrized test that iterates over
   each "one field missing/wrong" scenario would be more maintainable than seven
   individual tests as this grows.

### Observation

This is the packet that moves grouped-v2 from "shape is right" to "data is
complete." Before 344 the hot tuples referenced codebooks that only lived in
build-time memory. After 344 the codebooks are durable. That's a prerequisite
for any runtime execution. Well-bounded batched packet.
