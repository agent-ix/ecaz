# Task 67 Review Request: x86 SIMD Reviewer Follow-ups

## Scope

This packet covers follow-ups from the reviewer feedback on Task 67 packets
001-007.

Code commit under review:

- `9cb453f9d` - `Address RaBitQ x86 SIMD review follow-ups`

Current pushed head:

- `861cf49ee1872305aa2c91c6c14e88f4b89648d8`

## Changes

- Relaxed the bits=1 AVX-512 RaBitQ weighted byte-LUT kernels from
  `avx512f,avx512vpopcntdq` to `avx512f`; the implementation uses FMA over
  dequantized byte-LUT values and does not issue VPOPCNTDQ instructions.
- Updated dispatch safety comments and x86 feature-slot tests to reflect the
  AVX-512F-only bits=1 requirement.
- Made `backend_name()` preserve all AVX-512 feature combinations, including
  partial BF16 combinations that were previously collapsed.
- Added `ECAZ_SIMD=avx512+bf16` / `avx512_bf16` / `avx512f+bf16` override
  aliases.
- Switched the bits=4 and bits=8 batch estimator order tests from 6 candidates
  to 5 candidates so the odd tail path is covered by the compile target.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.log`.

Summary:

- `cargo fmt` completed successfully with the repository's existing stable
  rustfmt warnings about ignored nightly-only options.
- `git diff --check` passed.
- Focused compile-only validation passed for:
  - `x86_feature_slots_model_task67_kernel_requirements`
  - `backend_name_preserves_avx512_feature_parts`
  - `bits8_batch_estimator_matches_scalar_order`
  - `bits4_batch_estimator_matches_scalar_order`

Runtime unit execution remains intentionally skipped for this slice because the
local runtime path is still blocked by the unresolved PostgreSQL `LockBuffer`
symbol noted in prior Task 67 packets.
