# Review Request: SIMD Equivalence Test Suite

Commit: `a86157f` Add SIMD equivalence tests for production dims and edge cases

Scope:
- `src/quant/prod.rs` (3 new tests)
- `src/quant/hadamard.rs` (1 new test)

Summary:
- Audited existing SIMD-vs-scalar equivalence test coverage and identified four gaps
- Added four new tests to complete the equivalence test suite required by `plan/tasks/08-simd.md`

## Gap Analysis

Before this change, existing equivalence tests covered:
- `fwht_runtime_path_matches_scalar_on_random_inputs` — 1000 inputs, sizes 2^1..2^9 (up to 512)
- `fwht_tiled_avx2_exact_sizes_match_scalar_when_available` — deterministic data at 128..4096
- `srht_runtime_path_matches_scalar_on_random_inputs` — 1000 inputs, sizes 2^1..2^9
- `decode_eight_3bit_lanes_avx2_matches_scalar_when_available` — 1000 inputs
- `dispatched_score_matches_scalar_on_random_inputs` — 1000 inputs, dims 32/64/128/256

Gaps identified:
1. **No random data at production FWHT sizes** — tiled test only used one deterministic input
   per size at 1024/2048/4096
2. **No scoring equivalence at production dims** — the 4-accumulator AVX2 unroll was only
   tested at dims up to 256, never at 1024/1536/2048
3. **No tail-path coverage in scoring** — all tested dims were multiples of 32, so the 8-dim
   SIMD tail and scalar tail code paths were never exercised for the 3-bit AVX2 backend
4. **No QJL sign LUT validation** — the 256-entry static LUT was never verified against the
   bit-extraction reference

## New Tests

### `dispatched_score_matches_scalar_at_production_dims`
- 100 random inputs at dims 1024, 1536, 2048 with bits=4 (3-bit AVX2 path)
- Tolerance: `sqrt(dim) * 1e-6` — at dim=2048 the AVX2 4-accumulator tree reduction
  (acc0+acc1 + acc2+acc3) sums in a different order than scalar sequential accumulation,
  and FP non-associativity accumulates across ~2048 products
- Observed max relative errors: ~1.2e-6 at dim=1536, ~1.6e-5 at dim=2048

### `dispatched_score_matches_scalar_with_tail_dims`
- 1000 random inputs at dims 40, 100, 104, 108 with bits=4
- dim=40: 1×32 outer + 1×8 tail, no scalar tail
- dim=100: 3×32 outer + 0×8 tail + 4 scalar tail
- dim=104: 3×32 outer + 1×8 tail, no scalar tail
- dim=108: 3×32 outer + 1×8 tail + 4 scalar tail (all three paths)
- Uses 1e-6 tolerance (same as existing small-dim test)

### `qjl_sign_lanes_exhaustive`
- Validates all 256 entries in the compile-time `build_qjl_sign_lut()` table
- Each byte → 8 lanes verified against `(byte >> bit) & 1` reference

### `fwht_runtime_path_matches_scalar_at_large_sizes`
- 100 random inputs each at sizes 1024, 2048, 4096
- Covers the tiled AVX2 FWHT path with diverse data

## FP Tolerance Discussion

The sqrt(dim)-scaled tolerance for production dims is grounded in standard FP error analysis:
summing N random terms with different accumulation orders gives expected error ~sqrt(N) * eps.
The 4-accumulator tree reduction is mathematically equivalent but FP-nonequivalent to scalar
sequential accumulation. This is not a correctness concern — both paths produce valid estimates
within the quantizer's noise floor.

Please review:
- whether the sqrt(dim) scaling is the right tolerance model, or whether a fixed threshold
  (e.g., 5e-5) would be clearer
- whether any additional dims or bit-widths should be covered at production sizes
- whether the 100-iteration count for production dims is sufficient (reduced from 1000 to
  keep test runtime reasonable at high dims)
