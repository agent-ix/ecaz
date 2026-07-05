# Task 87 Source Scoring Map

This artifact records the current source surfaces inspected for the Phase 1
design. Line numbers are from commit `6f1703dab`.

## SPIRE

Primary leaf-column scoring site:

- `src/am/ec_spire/scan/candidates.rs:2445` allocates one score slot per V2
  column row.
- `src/am/ec_spire/scan/candidates.rs:2447` calls
  `SpirePreparedAssignmentScorer::score_batch_ip` with contiguous payload bytes,
  gamma column, and score output.
- `src/am/ec_spire/scan/candidates.rs:2457` walks scored rows in original row
  order and materializes candidates afterward.
- `src/am/ec_spire/scan/candidates.rs:2497` enters the selected-range/RaBitQ
  cutoff path, which stays out of Task 87's first TQ-only batch route.

Batch scorer implementation:

- `src/am/ec_spire/quantizer/mod.rs:297` defines
  `SpirePreparedAssignmentScorer::score_batch_ip`.
- `src/am/ec_spire/quantizer/mod.rs:331` loops over fixed-stride payload
  chunks.
- `src/am/ec_spire/quantizer/mod.rs:336` routes TurboQuant no-QJL 4-bit to
  `score_ip_from_parts_lut_no_qjl_4bit`.
- `src/am/ec_spire/quantizer/mod.rs:340` keeps the generic TurboQuant
  gamma-aware scorer as fallback.

Task 87 mapping:

- Replace the SPIRE-local batch loop with the shared `CandidateBatch` flush for
  the TurboQuant no-QJL 4-bit branch only.
- Keep the row-order score buffer contract unchanged so candidate visibility,
  deletion filtering, and heap append behavior remain SPIRE-owned.

## IVF

Prepared-query and scoring surface:

- `src/am/ec_ivf/quantizer.rs:164` defines `prepare_ip_query`.
- `src/am/ec_ivf/quantizer.rs:179` branches on `ExactScoreMode`.
- `src/am/ec_ivf/quantizer.rs:181` prepares the TurboQuant no-QJL 4-bit LUT
  query.
- `src/am/ec_ivf/quantizer.rs:237` defines `score_ip_from_parts`.
- `src/am/ec_ivf/quantizer.rs:253` matches
  `IvfPreparedQuery::TurboQuantNoQjl4BitLut`.
- `src/am/ec_ivf/quantizer.rs:261` calls
  `score_ip_from_parts_lut_no_qjl_4bit`.
- `src/am/ec_ivf/quantizer.rs:336` defines an existing RaBitQ batch helper,
  proving the IVF quantizer adapter already has a batch-shaped precedent.

Task 87 mapping:

- Add a shared batch scorer call around posting-list row chunks before IVF
  candidate heap/top-k insertion.
- Limit Task 87 to the `TurboQuantNoQjl4BitLut` branch; RaBitQ and grouped-PQ
  stay on their existing paths.

## HNSW

Traversal-local successor scoring:

- `src/am/ec_hnsw/graph.rs:1261` defines
  `load_successor_candidates_for_layer_with_storage`.
- `src/am/ec_hnsw/graph.rs:1275` computes valid neighbor tids for one layer.
- `src/am/ec_hnsw/graph.rs:1279` loops neighbor-by-neighbor.
- `src/am/ec_hnsw/graph.rs:1284` loads each neighbor graph element.
- `src/am/ec_hnsw/graph.rs:1285` invokes the score callback inline.
- `src/am/ec_hnsw/graph.rs:1288` pushes a scored beam candidate.

TurboQuant scoring function:

- `src/am/ec_hnsw/scan.rs:5005` gets the prepared full-LUT query.
- `src/am/ec_hnsw/scan.rs:5007` routes the full-LUT branch to
  `score_ip_from_parts_lut_no_qjl_4bit`.
- `src/am/ec_hnsw/scan.rs:5035` keeps the generic gamma-aware scorer fallback.

Task 87 mapping:

- Batch only within one successor expansion. The valid neighbor set is the
  natural batch boundary because greedy descent and beam search choose the next
  expansion from the current scored results.
- The batch id can be the neighbor `ItemPointer`; HNSW still owns deleted-row
  filtering, heap TID expansion, frontier ordering, and refill behavior.

## DiskANN

Current search codec and prefilter surface:

- `src/am/ec_diskann/quantizer.rs:29` defines `DiskannBuildCodec`.
- `src/am/ec_diskann/quantizer.rs:30` has the grouped-PQ build variant.
- `src/am/ec_diskann/quantizer.rs:34` has the RaBitQ build variant.
- `src/am/ec_diskann/quantizer.rs:111` maps those two variants to metadata
  search-code kinds.
- `src/am/ec_diskann/quantizer.rs:152` defines `DiskannPreparedPrefilter`.
- `src/am/ec_diskann/quantizer.rs:153` defines the binary-sidecar branch.
- `src/am/ec_diskann/quantizer.rs:157` defines the grouped-PQ branch.
- `src/am/ec_diskann/quantizer.rs:163` defines the RaBitQ branch.
- `src/am/ec_diskann/quantizer.rs:169` scores prefilter tuples by matching
  those branches. There is no TurboQuant prefilter branch in this enum.

Greedy scan scoring site:

- `src/am/ec_diskann/scan.rs:402` reads the entry tuple and scores it through
  the prefilter callback.
- `src/am/ec_diskann/scan.rs:434` iterates the picked node's neighbors.
- `src/am/ec_diskann/scan.rs:445` reads each neighbor tuple.
- `src/am/ec_diskann/scan.rs:446` scores each neighbor inline.
- `src/am/ec_diskann/scan.rs:462` repeats the same shape for the profiled
  scan path.

Task 87 mapping:

- This checkout cannot directly satisfy "DiskANN TurboQuant no-QJL 4-bit
  scoring site uses `CandidateBatch`" because DiskANN has no TQ search-code
  branch to route.
- If a TQ codec is added or found on a target branch, the batch boundary should
  be the decoded neighbor set for one expanded Vamana node, preserving the
  best-first control loop.
- Until that scope question is resolved, Phase 4 should not silently route
  grouped-PQ or RaBitQ through a TQ-only acceptance gate.

## pgvectorscale Reference

Reference file:

- `/home/peter/dev_bak/pgvectorscale/pgvectorscale/src/access_method/scan.rs`

Relevant resort-buffer lines:

- line 167 stores `resort_size`.
- line 168 stores `resort_buffer: BinaryHeap<ResortData>`.
- line 199 allocates the buffer with `BinaryHeap::with_capacity(resort_size)`.
- line 244 defines `next_with_resort`.
- line 255 fills the resort buffer until it reaches `resort_size`.
- line 259 fetches full distance for the candidate being resorted.
- line 279 pushes scored resort data into the heap.
- line 302 pops the best resorted item.

Task 87 copies the bounded-buffer shape, not the exact layer:

- pgvectorscale buffers result-iteration candidates and full-distance scores;
- Task 87 buffers compressed-code candidates before AM-owned frontier/top-k
  insertion.
