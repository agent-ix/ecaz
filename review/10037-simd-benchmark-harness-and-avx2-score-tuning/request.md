# Review Request: SIMD Benchmark Harness And AVX2 Score Tuning

Commit: `f80fbaf`

Scope:
- `src/bin/simd_bench.rs`
- `src/quant/prod.rs`
- `src/quant/simd.rs`
- `src/quant/mod.rs`
- `src/lib.rs`

Summary:
- add a lightweight `simd_bench` binary so the same executable can be run with
  `TQVECTOR_SIMD=scalar` or auto-detection for local scalar-vs-SIMD comparisons on any machine
- expose the selected backend name through the bench API and add process-level backend forcing in
  the runtime detector for reproducible per-process benchmarking
- tune the AVX2 scoring path by replacing per-lane QJL sign extraction with a packed-byte lookup
  table, reducing branchy sign expansion in the hot loop
- drop the SIMD dispatch for `score_ip_codes_lite` because the previous vectorized version was
  slower than scalar on the current x86_64 host

Local benchmark snapshot on this machine (`cargo run --release --bin simd_bench -- 5000`):
- scalar: `fwht/2048` `3697.7 ns`, `fwht/4096` `8019.6 ns`,
  `score_ip_encoded/d1536_b4` `17040.7 ns`, `score_ip_codes_lite/d1536_b4` `12828.1 ns`
- auto (`avx2+fma`): `fwht/2048` `3386.0 ns`, `fwht/4096` `7244.7 ns`,
  `score_ip_encoded/d1536_b4` `8198.3 ns`, `score_ip_codes_lite/d1536_b4` `13048.5 ns`

Please review:
- whether the new backend-forcing surface is narrow enough for benchmarking without leaking into
  normal runtime behavior
- whether the AVX2 score tuning still preserves the intended scalar semantics across the frozen API
- whether keeping the lite scorer on scalar until a demonstrably faster SIMD version exists is the
  right tradeoff for this B1 checkpoint
