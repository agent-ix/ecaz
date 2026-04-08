# Review Request: SIMD Runtime Dispatch And Equivalence

Commit: `b24139e`

Scope:
- `src/quant/hadamard.rs`
- `src/quant/prod.rs`
- `src/quant/simd.rs`
- `src/quant/mod.rs`

Summary:
- add cached runtime SIMD backend detection with AVX2+FMA on `x86_64`, NEON on `aarch64`, and
  scalar fallback elsewhere
- route `fwht_in_place`, `score_ip_encoded`, and the lite MSE scoring path through backend-aware
  dispatch without changing the public quantizer API surface
- keep preserved scalar implementations alongside the dispatched paths so module-local equivalence
  tests can compare scalar and dispatched behavior directly
- replace the old bit-by-bit MSE unpack helper with a bounded word-load extractor to reduce scalar
  overhead in both fallback and SIMD-assisted scoring loops
- add 1000-input random equivalence coverage for FWHT and the dispatched scoring entrypoints

Please review:
- whether the runtime dispatch boundaries stay within the frozen quantizer/scoring surface for B1
- whether the SIMD-assisted scoring paths preserve scalar semantics tightly enough across supported
  bit widths and dimensions
- whether the new backend-detection and target-feature usage looks safe on unsupported CPUs and
  leaves a clean path for future throughput-focused follow-up work
