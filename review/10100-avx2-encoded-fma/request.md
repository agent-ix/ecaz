# Review Request: AVX2 Encoded Scorer FMA

Commit: `16d88a7`

Scope:
- `src/quant/prod.rs`

Summary:
- replace separate multiply-plus-add pairs with `_mm256_fmadd_ps` in the common 3-bit AVX2
  encoded-scorer loop
- keep lite scoring, scalar fallback, generic-width logic, planner, scan runtime, graph traversal,
  and FWHT unchanged
- move this packet into the `10100` range to avoid extending the `1003x/1004x` review-number
  collision called out in `review/10041` feedback

Before/after AVX2 scorer snapshot on this machine (same harness at `5000` iterations):
- before (`0809555`, recorded in `review/10042`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `860.5 ns`, `score_ip_codes_lite/d1536_b4` `1358.8 ns`
- after (`16d88a7`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `789.5 ns`, `score_ip_codes_lite/d1536_b4` `1401.0 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations):
- auto (`avx2+fma`): `fwht/2048` `3391.9 ns`, `fwht/4096` `7004.3 ns`,
  `score_ip_encoded/d1536_b4` `790.6 ns`, `score_ip_codes_lite/d1536_b4` `1380.2 ns`

Experiment log:
- kept: `_mm256_fmadd_ps` in the encoded 3-bit AVX2 path only
- discarded before this checkpoint: applying FMA to `score_ip_codes_lite` regressed lite scoring on
  this host, so that variant was reverted before commit
- observed from the matched `5000`-iteration runs: `score_ip_encoded` improved by about `8%`
  versus `10042`, while lite scoring moved slightly the wrong way on the short run, consistent with
  the final code change not touching lite scoring
- observed from the longer `20000`-iteration rerun: the encoded scorer still landed about `11%`
  faster than `10042`, while lite stayed in the same band
- one initial `20000`-iteration pass came in noisier (`score_ip_encoded/d1536_b4` `919.0 ns`,
  `score_ip_codes_lite/d1536_b4` `1438.1 ns`); an immediate rerun settled lower, which reinforces
  the earlier reviewer note that `simd_bench` would benefit from a warmup pass before timed loops
- unchanged by design: scalar fallback and non-3-bit AVX2 logic remain as before

Please review:
- whether narrowing FMA to the encoded 3-bit AVX2 loop is the right boundary for this checkpoint
- whether the measured encoded-scorer win is strong enough to keep given the first noisy long run
- whether this packet records the kept and discarded FMA variants clearly enough for later B1 work
