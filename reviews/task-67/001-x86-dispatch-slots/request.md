# Task 67 Slice A: x86 Dispatch Slots

## Summary

Landed the Slice A plumbing-only checkpoint for RaBitQ Intel SIMD:

- `src/quant/simd.rs` now detects AVX-512 feature bits needed by Task 67: `avx512f`, `avx512vpopcntdq`, `avx512bw`, and `avx512bf16`.
- `src/quant/rabitq.rs` now has explicit x86 slot selection for future AVX-512 and AVX2 RaBitQ kernels across bits=1, bits=4, bits=8, and bf16 bits=4.
- The x86 RaBitQ slots intentionally return the scalar scoring path until Slices B-H add the actual `target_feature` kernel bodies.
- Existing non-RaBitQ SIMD users in `src/quant/hadamard.rs` and `src/quant/prod.rs` now treat AVX-512-capable hosts as eligible for the existing AVX2/FMA implementations.

Code commit: `19715a204e16a4b8142f2ad2ed95ebe3dc752647`

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings for unstable import-group settings.
- `cargo test -p ecaz quant::simd --no-run` completed successfully.
- Runtime test execution was attempted for `quant::simd` and the new RaBitQ x86 slot test, but the local test binary failed to start with unresolved PostgreSQL symbol `LockBuffer` before test bodies ran.

## Review Focus

- Confirm the new `SimdBackend::Avx512` shape is acceptable for later kernel registration.
- Confirm the x86 RaBitQ slot map covers Task 67 kernels without changing the shared estimate bodies.
- Confirm routing AVX-512-capable hosts through existing AVX2/FMA non-RaBitQ paths is acceptable until dedicated kernels exist.
