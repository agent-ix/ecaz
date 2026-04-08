# Review Request: AVX2 3-Bit LUT Gather

Commit: `f1cb16f`

Scope:
- `src/quant/prod.rs`

Summary:
- keep the aligned 3-bit decode and AVX2 lane accumulators from `10038` and `10039`
- replace the remaining scalar LUT fill in the hot 3-bit AVX2 scorer loop with
  `_mm256_i32gather_ps`, so the common 4-bit quantizer path now stays vectorized through both
  lookup and accumulation
- leave scalar scoring, generic-width AVX2 handling, FWHT, planner wiring, and scan runtime
  untouched

Before/after AVX2 scorer snapshot on this machine (same harness at `5000` iterations):
- before (`ea41ff5`, recorded in `review/10039`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1597.0 ns`, `score_ip_codes_lite/d1536_b4` `2104.2 ns`
- after (`f1cb16f`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1253.0 ns`, `score_ip_codes_lite/d1536_b4` `2204.1 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations):
- auto (`avx2+fma`): `fwht/2048` `4388.9 ns`, `fwht/4096` `8971.4 ns`,
  `score_ip_encoded/d1536_b4` `1163.1 ns`, `score_ip_codes_lite/d1536_b4` `2220.4 ns`

Experiment log:
- kept: AVX2 gather-based LUT loads for the 3-bit scorer path in `score_ip_from_split_parts_avx2`
- observed from the matched `5000`-iteration runs: `score_ip_encoded` improved by about `21.5%`
  over `10039` on the same host/harness
- observed from the longer `20000`-iteration run: `score_ip_encoded` improved further to
  `1163.1 ns`, so the gather change still looks stable outside the shorter sample
- unchanged by design: scalar scorer behavior and non-3-bit AVX2 handling
- not pursued in this slice: wider gather work for non-3-bit quantizers, because B1 is still
  driven by the common 4-bit frozen surface on this host

Please review:
- whether using `_mm256_i32gather_ps` for the 3-bit AVX2 hot loop is a good tradeoff versus the
  previous scalar LUT fill
- whether keeping the generic-width path untouched is the right narrow checkpoint boundary
- whether the packet keeps enough benchmark history to reconstruct which SIMD micro-slices moved
  the scorer materially
