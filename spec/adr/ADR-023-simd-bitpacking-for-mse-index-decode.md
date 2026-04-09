---
id: ADR-023
title: "SIMD Bit-Packing for MSE Index Decode in Scoring Hot Path"
status: PROPOSED
impact: Affects FR-014, NFR-001, B1 SIMD task
date: 2026-04-08
---
# ADR-023: SIMD Bit-Packing for MSE Index Decode in Scoring Hot Path

## Context

The scoring hot path in `score_ip_from_split_parts` calls `mse_index_at` once per dimension to
extract a sub-byte MSE centroid index from the packed candidate payload. The current implementation
(`read_bits_le` in `src/quant/prod.rs`) decodes one bit at a time in a scalar loop.

At 1536 dimensions and 3-bit indices, this means 1536 calls to `mse_index_at`, each doing 3
iterations of bit extraction. That is 4608 bit-level operations per candidate scored — a
significant fraction of the scoring cost.

The same pattern applies to `qjl_sign_at`, which extracts one sign bit per dimension (1536
single-bit reads per candidate).

### Existing ecosystem

The `bitpacking` crate (by the Tantivy author) provides SIMD-accelerated packed integer decoding
for exactly this class of problem: extracting many small-width integers from a packed byte stream.
It supports SSE2, AVX2, and scalar fallbacks, and handles bit-widths from 1-32.

### Integration considerations

- The MSE index width is `bits - 1` (3 bits at the current `bits = 4` config), which is within
  `bitpacking`'s supported range.
- QJL sign bits are 1-bit packed, which `bitpacking` also handles.
- The packed byte layout must match `bitpacking`'s expected format (little-endian, contiguous). The
  current `write_bits_le` / `read_bits_le` uses a compatible LE bit-ordering.
- `bitpacking` decodes in blocks of 128 or 256 integers. At 1536 dimensions, that is 6-12 blocks
  — clean alignment.

### Alternative: hand-rolled SIMD decode

Instead of depending on `bitpacking`, the decode could be hand-rolled using AVX2 shift/mask
operations. This avoids a dependency but requires implementing and maintaining sub-byte SIMD
extraction for each supported bit-width.

## Hypothesis

Replacing the bit-at-a-time `mse_index_at` / `qjl_sign_at` calls with block-decoded integer
arrays (via `bitpacking` or hand-rolled SIMD) will:

1. Eliminate per-dimension function call overhead in the scoring loop
2. Enable the scoring loop to operate on contiguous decoded index arrays, improving autovectorization
   and explicit SIMD opportunities
3. Combine naturally with ADR-022 (direct multiply): decode a block of indices, gather codebook
   entries, FMA with query values — all in registers

## What Not To Assume

1. **Do not assume `bitpacking` is the only option.** Hand-rolled SIMD decode may be simpler if
   tqvector only needs 1-bit and 3-bit extraction. Evaluate both.
2. **Do not assume decode is the bottleneck.** Profile first — if scoring is dominated by cache
   misses (LUT or page reads), faster decode alone won't help much.
3. **Do not change the on-disk packed format.** The decode optimization must work with the existing
   `write_bits_le` byte layout. Format changes would require a migration.

## Required Validation

1. **Profile:** Measure what fraction of `score_ip_encoded` time is spent in `mse_index_at` /
   `read_bits_le` vs the accumulation loop.
2. **Prototype:** Decode a block of 256 3-bit indices using `bitpacking` or hand-rolled AVX2, then
   score using the decoded array. Measure throughput vs the current per-dimension decode.
3. **Integration:** Confirm the existing packed byte layout is compatible with block decode without
   format changes.

## Decision

**Open.** Investigate as part of B1 SIMD scoring work, ideally in combination with ADR-022. The
two ADRs are complementary: ADR-022 changes *what* the scoring loop computes, ADR-023 changes
*how* it reads its inputs.

## Consequences

### If `bitpacking` is adopted
- New dependency added to `Cargo.toml`
- Scoring loop changes from per-dimension `mse_index_at` to block-decoded `&[u16]` or `&[u32]`
  slices
- Must verify `bitpacking`'s bit layout matches tqvector's existing packed format

### If hand-rolled SIMD decode is preferred
- No new dependency
- More maintenance surface for sub-byte SIMD extraction
- Tighter integration with the scoring SIMD loop (decode and score can be fused)

### If rejected
- Keep the current bit-at-a-time decode
- Document that decode cost is not the bottleneck (if profiling confirms)
