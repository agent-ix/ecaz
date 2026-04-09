# Task 08: SIMD Acceleration

Status: **in progress** — AVX2+FMA complete, NEON 3-bit specialization remaining

## Scope

Implement SIMD-accelerated versions of performance-critical functions on x86_64 (AVX2+FMA) and aarch64 (NEON) with scalar fallback and runtime detection.

## Subtasks

- [x] **AVX2+FMA implementations.** `fwht` (tiled), `score_ip_encoded` (4-accumulator unroll with codebook permute), `score_ip_codes_lite` (4-accumulator unroll), `decode_eight_3bit_lanes_avx2`, `qjl_sign_lanes` LUT. 3-bit path also skips LUT build in `prepare_ip_query`.
- [ ] **NEON implementations.** NEON scoring path exists but lacks 3-bit specialization (still uses scalar `mse_index_at` per lane). FWHT does not have NEON. Requires aarch64 hardware for benchmarking.
- [x] **Runtime feature detection.** `SimdBackend` enum in `src/quant/simd.rs` with cached detection via `OnceLock`.
- [x] **Equivalence tests.** Suite covers: production dims (1024/1536/2048) with sqrt(dim)-scaled tolerance, tail dims (40/100/104/108), QJL sign LUT exhaustive, FWHT at large sizes (1024/2048/4096). Original 1e-6 threshold holds for small dims; production dims use `sqrt(dim)*1e-6` due to FP accumulation order differences in 4-accumulator tree reduction.
- [x] **Throughput benchmark.** FWHT AVX2 achieves 4.4x scalar at dim=2048 (exceeds 3x target). Cumulative gains: `prepare_ip_query` 83% faster, `score_ip_encoded` 34% faster, `score_ip_codes_lite` 52% faster.

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
- Do not merge SIMD into main until after Task 05 A3 confirms scalar scan correctness. Keep on a feature branch until then.
- FWHT butterfly in AVX2 is the most complex piece (cross-lane shuffles).
- Variable bit-width MSE index unpacking (bits 2-8) in SIMD requires careful masking.
- Testing NEON requires access to an aarch64 machine (e.g., AWS Graviton).
