# Task 08: SIMD Acceleration

Status: not started — **can start immediately (no dependencies on critical path)**

## Scope

Implement SIMD-accelerated versions of performance-critical functions on x86_64 (AVX2+FMA) and aarch64 (NEON) with scalar fallback and runtime detection.

## Subtasks

- [ ] **AVX2+FMA implementations.** `fwht`, `score_ip_encoded`, `score_ip_encoded_lite`, `qjl_bit_expand`.
- [ ] **NEON implementations.** Same four functions.
- [ ] **Runtime feature detection.** `is_x86_feature_detected!` / `is_aarch64_feature_detected!` with cached result at first call.
- [ ] **Equivalence tests.** SIMD vs scalar within 1e-6 relative error on 1000 random inputs for each function.
- [ ] **Throughput benchmark.** fwht AVX2 >= 3x scalar at dim=2048.

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
