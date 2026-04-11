# Task 08: SIMD Acceleration

Status: done on `main` — x86_64 merged and validated; aarch64 runtime validation still requires hardware

## Scope

Implement SIMD-accelerated versions of performance-critical functions on x86_64 (AVX2+FMA) and aarch64 (NEON) with scalar fallback and runtime detection.

## Subtasks

- [x] **AVX2+FMA implementations.** `fwht`, `score_ip_encoded`, `score_ip_codes_lite`, padded SRHT query prep, and 3-bit decode/scoring specializations are merged on `main`.
- [x] **NEON implementations.** 3-bit encoded/code-to-code scorers and the sign-lane fix are merged on `main`; runtime validation still requires aarch64 hardware.
- [x] **Runtime feature detection.** Cached runtime dispatch lives in `src/quant/simd.rs` with `TQVECTOR_SIMD` override support.
- [x] **Equivalence tests.** Coverage now includes production dims, tail dims, large FWHT sizes, and exhaustive QJL sign-lane validation.
- [x] **Throughput benchmark.** Current validation on the merged branch shows `fwht/2048` at about `895 ns` with `avx2+fma` vs `2909 ns` scalar (`~3.25x` faster); direct current-main vs merged Criterion checks show `prepare_ip_query/d1536_b4` improving from about `21.9 us` to `5.46 us` and `score_ip_encoded/d1536_b4` from about `6.34 us` to `1.38 us`.

## Owns

- `FR-014`

## Dependencies

- None — scalar APIs are frozen (Tasks 01-03 complete)

## Unblocks

- Performance targets in NFR-001

## Deliverables

- SIMD implementations with `#[target_feature(enable = "...")]` attributes
- Runtime dispatch layer
- Scalar fallback (existing code, no changes needed)
- Equivalence test suite
- Throughput benchmark

## Primary Tests

- `TC-016`, `TC-017`, `TC-030`: SIMD correctness
- `BC-008`: SIMD throughput

## Notes

- This task can run on a **separate parallel agent** with no coordination required.
- A3 and A4 are now closed, so the SIMD work is merged on `main`.
- FWHT butterfly in AVX2 is the most complex piece (cross-lane shuffles).
- Variable bit-width MSE index unpacking (bits 2-8) in SIMD requires careful masking.
- Testing NEON requires access to an aarch64 machine (e.g., AWS Graviton).

## Scoring Hot Path Investigation (ADR-022 + ADR-023)

The following two ADRs identify potential improvements to the `score_ip_encoded` / `score_ip_from_split_parts` hot path that should be investigated as part of B1 SIMD scoring work:

1. **ADR-022: Drop the scoring LUT for direct codebook multiply.**
   TurboQuant uses only 8 centroids (3-bit MSE at `bits=4`). The current LUT precomputes `dim * 8` floats (48-64 KB at 1536/2048-dim), which pushes or exceeds L1 cache. With 8 centroids the codebook is 32 bytes (one cache line). Direct `codebook[index] * rotated_query[dim]` eliminates the LUT allocation, improves L1 residency during multi-candidate scoring, and enables straightforward `_mm256_fmadd_ps` vectorization instead of expensive `_mm256_i32gather_ps` LUT reads. Quality is identical — same computation, different order.

2. **ADR-023: SIMD bit-packing for MSE index decode.**
   The current `mse_index_at` / `read_bits_le` decodes one bit at a time per dimension (4608 bit ops per candidate at 1536-dim, 3-bit). Block-decoding via the `bitpacking` crate or hand-rolled AVX2 shift/mask would batch 128-256 index extractions, feeding the scoring loop contiguous decoded arrays. Evaluate whether `bitpacking` is worth the dependency vs hand-rolled decode for just 1-bit and 3-bit widths.

These two ADRs are complementary: ADR-022 changes what the scoring loop computes, ADR-023 changes how it reads packed inputs. Together they could turn the scoring inner loop into: block-decode indices → gather codebook entries → FMA with query values, all in registers.

Recommended investigation order:
1. Profile `score_ip_encoded` to establish baseline and confirm where time is spent (LUT access vs decode vs accumulation)
2. Prototype direct-multiply scoring (ADR-022) and benchmark against LUT path
3. Prototype block decode (ADR-023) and measure combined effect
4. Fuse MSE accumulation and QJL sign accumulation into a single pass
