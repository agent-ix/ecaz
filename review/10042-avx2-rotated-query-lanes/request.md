# Review Request: AVX2 Rotated Query Lanes

Commit: `0809555`

Scope:
- `src/quant/prod.rs`

Summary:
- extend `PreparedQuery` with cached rotated-query lanes for the frozen inner-product scorer
- keep the existing scalar and generic-width surfaces, but for the common 4-bit AVX2 path replace
  large-table LUT gathers with the same 8-entry codebook permute pattern already used by the lite
  scorer
- leave planner, scan runtime, graph traversal, and FWHT code untouched

Before/after AVX2 scorer snapshot on this machine (same harness at `5000` iterations):
- before (`beded2c`, recorded in `review/10041`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1167.5 ns`, `score_ip_codes_lite/d1536_b4` `1364.2 ns`
- after (`0809555`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `860.5 ns`, `score_ip_codes_lite/d1536_b4` `1358.8 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations):
- auto (`avx2+fma`): `fwht/2048` `3417.8 ns`, `fwht/4096` `7171.7 ns`,
  `score_ip_encoded/d1536_b4` `888.5 ns`, `score_ip_codes_lite/d1536_b4` `1397.5 ns`

Experiment log:
- kept: cached rotated-query lanes in `PreparedQuery` and AVX2 codebook-permute scoring for the
  3-bit encoded path
- observed from the matched `5000`-iteration runs: `score_ip_encoded` improved by about `26%`
  over `10041`, while lite scoring stayed essentially flat
- observed from the longer `20000`-iteration run: the encoded-scorer win held in the same band,
  so this does not look like a short-run artifact
- unchanged by design: scalar scoring still uses the existing `prepared.lut` surface, and non-3-bit
  AVX2 paths still use the generic logic
- discarded before this checkpoint: a 16-dimension unroll of the 3-bit AVX2 loop and a sign-mask
  XOR replacement for the QJL multiply; both regressed on this host and were reverted

Please review:
- whether caching the rotated-query lanes on `PreparedQuery` is an acceptable frozen-surface
  expansion for B1
- whether the encoded-scorer AVX2 path now has the right specialization boundary for the common
  4-bit quantizer case
- whether the packet documents the discarded experiments clearly enough for later B1 work
