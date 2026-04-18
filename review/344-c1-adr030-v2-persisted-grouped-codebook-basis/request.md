# Review Request: C1 ADR-030 V2 Persisted Grouped Codebook Basis

## Context

Packet `343` shared the grouped-PQ packed-code decode and f32 LUT scoring primitive between the
study harness and the runtime lane.

That made the next real runtime blocker clearer:

1. grouped-v2 hot tuples already persist grouped search codes
2. the study harness can already build grouped query LUTs from grouped codebooks
3. but the build/runtime path still did **not** persist the learned grouped codebooks anywhere on
   disk

Without persisted grouped codebooks, the future grouped runtime scorer would have no trustworthy
runtime source for its query LUT rows.

## Problem

ADR-030 v2 now had a shared grouped scorer primitive, but not the persisted model state required to
use it in the actual index runtime.

Concretely, the branch was missing all of these together:

1. a metadata pointer to the grouped codebook payload
2. an on-disk tuple contract for grouped codebook pages
3. builder-side persistence of grouped codebooks into the grouped-v2 index
4. a read helper that can load those codebooks back from a real grouped-v2 build
5. a shared grouped query-LUT builder that runtime and study code can both target

## Batched Slice

This checkpoint intentionally batches the closely related prerequisites above into one review packet:

1. add a shared grouped-PQ query LUT builder
2. persist grouped codebooks in the grouped-v2 build output
3. store a metadata pointer to the persisted grouped codebook chain
4. add graph/read helpers that load the persisted grouped codebooks back into runtime memory
5. keep the grouped-v2 external scan gate unchanged

This still excludes:

- no grouped-v2 ordered scan enablement
- no grouped candidate-score cutover to the shared grouped LUT path yet
- no planner/runtime gate lift

## Implementation

Updated shared quant helpers:

- `src/quant/grouped_pq.rs`
- `src/lib.rs`
- `src/bin/approx_score_study.rs`

New shared helper:

- `build_grouped_pq_lut_f32(rotated_query, flat_codebooks, group_size)`

Behavior:

1. builds grouped f32 LUT rows from one flat persisted codebook layout
2. keeps the existing shared nibble decode and score helper as the downstream scorer
3. routes the grouped-PQ study harness through the same shared LUT builder instead of a private
   copy

Updated storage/runtime boundary:

- `src/am/page.rs`
- `src/am/build.rs`
- `src/am/graph.rs`
- `src/am/scan.rs`
- `src/am/insert.rs`
- `src/am/vacuum.rs`
- `src/lib.rs`

New persistence/read pieces:

1. metadata now stores `grouped_codebook_head`
2. added grouped codebook tuple tag + page codec:
   - `TqGroupedCodebookTuple`
   - `TqGroupedCodebookTupleRef`
3. added `DataPage` / `DataPageChain` insert-read-update helpers for grouped codebook tuples
4. extended `V2GroupedBuildPlan` to retain the trained grouped model
5. added builder helper `stage_v2_grouped_codebook_tuples(...)`
6. made `grouped_v2_flush_output(...)` append persisted grouped codebook tuples and publish their
   head pointer in metadata
7. strengthened `GraphStorageDescriptor::from_metadata(...)` so grouped-v2 metadata must advertise
   a persisted grouped codebook chain
8. added grouped graph helper `load_grouped_codebook_model(...)`
9. added grouped borrowed read helper `with_grouped_codebook_tuple(...)`

## Measurements

This packet is persistence/runtime-basis work, so there are no new recall or latency measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test build_grouped_pq_lut_f32_uses_flat_codebooks_by_group --lib`: passed
  - `cargo test grouped_codebook_tuple_roundtrip --lib`: passed
  - `cargo test stage_v2_grouped_codebook_tuples_links_groups_in_order --lib`: passed
  - `cargo test graph_storage_descriptor_rejects_grouped_v2_missing_codebook_head --lib`: passed
  - `cargo test grouped_pq_u8_score_tracks_f32_for_same_code --bin approx_score_study`: passed
  - `cargo test test_grouped_v2_graph_reads_load_persisted_codebooks --no-default-features --features 'pg17 pg_test'`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 now persists the grouped model state needed by a future runtime grouped scorer.

What this de-risks:

1. grouped-v2 search codes are no longer orphaned from the codebooks required to score them
2. runtime work can now load the actual persisted grouped codebooks instead of depending on
   build-only memory
3. the study harness and runtime lane now share the same grouped query-LUT builder shape
4. grouped-v2 metadata now fails fast if the persisted grouped codebook contract is incomplete

## Next Slice

The next runtime batch should use this persisted model state inside scan-owned grouped query prep:

1. load grouped codebooks during grouped-v2 scan setup
2. build and cache grouped query LUT rows from scan-owned prepared query state
3. wire the shared grouped f32 scorer into grouped candidate scoring while still keeping the
   external grouped-v2 runtime gate in place
