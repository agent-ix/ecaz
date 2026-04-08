# Review Request: AVX2 Lite Codebook Permute

Commit: `beded2c`

Scope:
- `src/quant/prod.rs`

Summary:
- add an AVX2 fast path for `score_ip_codes_lite` / `score_ip_encoded_lite` on the common
  4-bit quantizer surface
- reuse the aligned 3-bit decode helper, but select codebook values with
  `_mm256_permutevar8x32_ps` from one loaded 8-lane codebook vector instead of staying on the
  scalar path
- keep scalar behavior for non-3-bit widths and for non-AVX2 hosts

Before/after AVX2 scorer snapshot on this machine (same harness at `5000` iterations):
- before (`f1cb16f`, recorded in `review/10040`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1253.0 ns`, `score_ip_codes_lite/d1536_b4` `2204.1 ns`
- after (`beded2c`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1167.5 ns`, `score_ip_codes_lite/d1536_b4` `1364.2 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations, rerun after the
checkpoint landed cleanly):
- auto (`avx2+fma`): `fwht/2048` `4386.1 ns`, `fwht/4096` `9071.5 ns`,
  `score_ip_encoded/d1536_b4` `1165.6 ns`, `score_ip_codes_lite/d1536_b4` `1378.7 ns`

Experiment log:
- kept: AVX2 lite-scorer dispatch using 8-lane codebook permutes for the 3-bit path
- observed from the matched `5000`-iteration runs: `score_ip_codes_lite` improved by about `38%`
  over `10040`, so the old “lite scorer stays scalar” decision is no longer correct on this branch
- observed from repeated `20000`-iteration runs: the lite-scorer win held at roughly
  `1.38 us`, while `score_ip_encoded` stayed in the same general band as the prior checkpoint
- unchanged by design: scalar code-to-code scoring for non-3-bit widths and all non-AVX2 hosts
- superseded earlier branch assumption: the previous review packets kept lite scoring on scalar
  because the first SIMD attempt lost; this checkpoint replaces that decision with a narrower AVX2
  path that is benchmark-positive on this host

Please review:
- whether the AVX2 permute-based lite scorer is the right narrow reintroduction of SIMD dispatch
  for `score_ip_codes_lite`
- whether keeping the dispatch limited to the 3-bit/4-bit surface is the right checkpoint
  boundary
- whether the packet documents the reversal of the earlier “scalar only” decision clearly enough
