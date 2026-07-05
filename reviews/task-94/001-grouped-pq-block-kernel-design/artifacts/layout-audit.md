# Grouped-PQ Layout Audit

## Existing Scorer

`src/quant/grouped_pq.rs` is the pre-kernel scalar reference. The logical LUT
layout is row-major `[group][centroid]` with 16 centroids per group:

```text
score = sum(group = 0..group_count) {
    lut_f32[group * 16 + nibble(code, group)]
}
```

Candidate codes are packed 4-bit centroid indices. Even groups use the low
nibble, odd groups use the high nibble. The required payload length is
`group_count.div_ceil(2)`.

The scalar block kernel must preserve this exact accumulation order per
candidate so Phase 2 can assert `f32::to_bits()` parity against
`grouped_pq_score_f32`.

## Shape Inputs

The block scorer needs only:

- `lut_f32: &[f32]`
- `group_count: usize`
- `codes: [&[u8]; 32]`
- `out_scores: &mut [f32]`

`group_size` is not used by scoring after the LUT has been prepared, but it
remains part of `QuantSearchCodecTag::GroupedPq` and AM shape validation.

Valid shapes:

- `lut_f32.len() == group_count * 16`
- every code has length at least `group_count.div_ceil(2)`
- `out_scores.len() >= 32`
- AM metadata uses `CandidateMeta::GroupedPq { group_count }`

Shape failures must return before recording counters.

## AM Surfaces

IVF:

- Prepared query: `IvfPreparedQuery::PqFastScan { lut, group_count, suffix_max }`.
- Scalar path: `IvfQuantizer::score_ip_from_parts` -> `grouped_pq_score_f32`.
- Current `IvfQuantCodec::score_ip_batch` overrides only TurboQuant LUT32; grouped-PQ falls through to per-candidate scalar.
- Task 94 registration should add a grouped-PQ branch in `IvfQuantCodec::score_ip_batch` and keep min-bound scalar pruning unchanged.

DiskANN:

- Prepared query: `DiskannPreparedPrefilter::GroupedPq { query_lut, group_count, group_size, ... }`.
- Scalar path: `DiskannGroupedPqPrefilterCodec::score_ip_candidate` -> `grouped_pq_score_f32`.
- Current `DiskannPreparedPrefilter::score` scores a single tuple. Phase 6 should add a batch bridge for the scan path while preserving the sign convention: prefilter returns negative inner product for distance ordering.

HNSW:

- Prepared query: `PreparedGroupedScanQuery { lut_f32, group_count, group_size, search_code_len, ... }`.
- Scalar path: `HnswGroupedPqScanCodec::score_ip_candidate` -> `grouped_pq_score_f32`.
- Task 94 says HNSW where applicable. HNSW has a grouped-PQ `QuantCodec`, so Phase 6 should register it if the scan loop has a natural candidate-batch boundary. If not, the closeout must document the missing batch boundary and leave only scalar candidate scoring for HNSW.

## Module Name

The task file names `src/quant/pq_fastscan32/`, while the lane prompt names
`src/quant/grouped_pq_block/`. This packet proposes
`src/quant/grouped_pq_block/{mod.rs,scalar.rs,neon.rs,sve.rs,avx2.rs}` because
it matches the coder-lane isolation rule and the `QuantCodecKind::GroupedPq`
label. Public functions can still use `grouped_pq`/`pq_fastscan` terminology in
doc comments where that clarifies the storage format.
