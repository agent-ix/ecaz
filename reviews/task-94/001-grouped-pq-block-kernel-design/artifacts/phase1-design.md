# Task 94 Phase 1 Design

This packet proposes the canonical grouped-PQ/PqFastScan 32-wide block kernel
under ADR-076. No code is included in Phase 1.

## Contract

- Module path: `src/quant/grouped_pq_block/{mod.rs,scalar.rs,neon.rs,sve.rs,avx2.rs}`.
- Block width: 32 candidates.
- Public dispatch: `score_grouped_pq_block32(lut_f32, group_count, codes, out_scores) -> Isa`.
- ISA functions:
  - `score_block32_scalar(...) -> Isa`
  - `score_block32_neon(...) -> Isa`
  - `score_block32_sve(...) -> Isa`
  - `score_block32_avx2(...) -> Isa`
- Unsupported ISA stubs delegate to scalar and return `Isa::Scalar`.
- Batch entry remains `QuantCodec::score_ip_batch`; AMs do not call ISA modules.
- Width gating: `batch.len() >= 32` scores whole 32-candidate blocks, then scalar tail.
- Shape failures happen before counters.

## Scalar Reference

The scalar block kernel mirrors `grouped_pq_score_f32` exactly, including
candidate-local accumulation order:

```text
scores[0..32] = 0.0
for candidate in 0..32:
    acc = 0.0
    for group in 0..group_count:
        byte = codes[candidate][group / 2]
        centroid = if group is even { byte & 0x0f } else { byte >> 4 }
        acc += lut_f32[group * 16 + centroid]
    scores[candidate] = acc
```

Phase 2 parity test:

- generate grouped-PQ LUTs and packed codes for group counts covering common
  shapes 8, 16, 32, and one non-multiple-of-2 tail shape if existing metadata
  permits it;
- compare every scalar block output against `grouped_pq_score_f32`;
- assert `score.to_bits() == reference.to_bits()`.

## NEON Strategy

NEON operates on four candidates at a time while preserving a deterministic
per-candidate group loop:

```text
for candidates in chunks of 4 within block32:
    acc4 = f32x4(0)
    for group in 0..group_count:
        packed4 = load the code byte for the four candidates
        idx4 = select low/high nibble for this group
        lut16 = load row group*16 as four f32x4 tables or scalar-gather lanes
        vals4 = gather lut16[idx4]
        acc4 += vals4
    store acc4
```

For the first NEON landing, a scalar lane gather from the 16-entry f32 row is
acceptable if the surrounding loop, code-byte extraction, and accumulation are
vectorized. If `tbl` is used, it should operate on a local byte/index staging
format and widen back to f32 without changing the scoring semantics.

Expected tolerance: <=4 ULP or <=1e-6 relative versus scalar reference.

## SVE2 Strategy

SVE2 must be vector-length agnostic. The kernel processes candidates in
predicate-sized chunks until 32 lanes are complete:

```text
candidate_base = 0
while candidate_base < 32:
    pg = whilelt(candidate_base, 32)
    acc = f32 lanes zeroed under pg
    for group in 0..group_count:
        byte_indices = candidate_base..candidate_base+VL
        packed = gather candidate code byte group/2 under pg
        idx = select low/high nibble under pg
        vals = gather lut_f32[group*16 + idx] under pg
        acc += vals
    store acc under pg
    candidate_base += runtime_vector_lanes
```

On Graviton 4, dispatch must report `Isa::Sve2`; the packet manifest must
record the measured runtime SVE vector length exactly as observed, for example
`sve2-128` only if that is what the host reports.

Expected tolerance: <=4 ULP or <=1e-6 relative versus scalar reference.

## AVX2 Strategy

AVX2 processes eight candidates per vector:

```text
for candidates in chunks of 8 within block32:
    acc8 = f32x8(0)
    for group in 0..group_count:
        packed8 = gather/load byte group/2 for eight candidate code pointers
        idx8 = select low/high nibble
        lut_low = load row entries 0..7
        lut_high = load row entries 8..15
        vals8 = gather lut row by idx8, using _mm256_i32gather_ps or staged shuffle
        acc8 += vals8
    store acc8
```

The preferred first AVX2 implementation is `_mm256_i32gather_ps` against the
16-entry f32 LUT row because the row is small and this minimizes layout risk.
If gather overhead dominates, a later packet can add a repacked byte or fixed
point LUT. That would be a separate design change because scalar parity is
anchored on f32 LUT accumulation.

Expected tolerance: <=4 ULP or <=1e-6 relative versus scalar reference.

## QuantCodec Fit

Task 94 should add a grouped-PQ batch helper in `src/am/common/candidate_batch.rs`
following the existing LUT32 timing pattern:

```text
score_grouped_pq_batch_for(surface, prepared_lut, group_count, batch, out):
    validate out len
    validate lut len == group_count * 16
    if batch.len() < 32:
        time scalar loop using grouped_pq_score_f32
        record scalar under (surface, grouped_pq, scalar)
        return
    for each full block:
        validate each payload meta and code len
        isa = grouped_pq_block::score_grouped_pq_block32(...)
        record kernel under (surface, grouped_pq, isa)
    scalar tail:
        grouped_pq_score_f32 for remaining candidates
        record via record_block_scalar_score_for(surface, grouped_pq, ...)
```

AM registration:

- IVF: override the grouped-PQ arm in `IvfQuantCodec::score_ip_batch`.
- DiskANN: add a scan-local candidate batch path for grouped-PQ prefilter rows
  and preserve the existing negative-score convention at the caller boundary.
- HNSW: register `HnswGroupedPqScanCodec::score_ip_batch` if the scan loop can
  naturally form candidate batches; otherwise document scalar-only HNSW in the
  Phase 6 packet and leave no direct ISA calls.

## Counter Contract

- Kernel rows: `record_block_kernel_score` under
  `(surface, QuantCodecKind::GroupedPq, dispatched_isa)`.
- Scalar tails and batches below width 32:
  `record_block_scalar_score_for(surface, QuantCodecKind::GroupedPq, ...)`,
  which records `isa=scalar`.
- A scalar ISA fallback for a full block is still a kernel row with
  `isa=scalar` and `kernel_candidates=32`, matching ADR-076 fallback semantics.
- Counter evidence in measurement packets must include direct
  `[block-kernel-counters]` rows, not only `[task87-counters]`.

## Phase Plan

Phase 2: add scalar module and bit-exact tests against `grouped_pq_score_f32`.

Phase 3: add NEON kernel and tolerance tests; capture Graviton 4 dispatch fact
only when running on a G4 host.

Phase 4: add vector-length-agnostic SVE2 and record measured runtime vector
length in the packet manifest.

Phase 5: add AVX2 and Intel host tolerance evidence.

Phase 6: register IVF, DiskANN, and HNSW where applicable through
`QuantCodec::score_ip_batch`; close the latency-suite direct
`[block-kernel-counters]` emission gap.

Phase 7: close out with per-AM x ISA results table, ADR-076 pointer, and task
status update.
