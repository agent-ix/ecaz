# Review Request: Aligned 3-Bit Decode Specialization

Commit: `3adcc2d`

Scope:
- `src/quant/prod.rs`

Summary:
- specialize the common `bits_per_index == 3` path used by the frozen 4-bit quantizer surface
  so `score_ip_encoded` and `score_ip_codes_lite` can decode eight packed centroid indices from
  one aligned 24-bit load instead of calling the generic per-dimension unpacker
- keep the generic scalar fallback for all other bit widths and keep the existing AVX2 dispatch
  surface unchanged apart from reusing the aligned 3-bit decoder in the hot loop
- add a direct unit test that locks the aligned 3-bit helper against the existing packer

Before/after scorer snapshot on this machine (same harness at `5000` iterations):
- before (`f80fbaf`, recorded in `review/10037`, scalar):
  `score_ip_encoded/d1536_b4` `17040.7 ns`, `score_ip_codes_lite/d1536_b4` `12828.1 ns`
- before (`f80fbaf`, recorded in `review/10037`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `8198.3 ns`, `score_ip_codes_lite/d1536_b4` `13048.5 ns`
- after (`3adcc2d`, rerun for this packet, scalar):
  `score_ip_encoded/d1536_b4` `3644.8 ns`, `score_ip_codes_lite/d1536_b4` `2145.5 ns`
- after (`3adcc2d`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1952.0 ns`, `score_ip_codes_lite/d1536_b4` `2145.6 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations):
- scalar: `fwht/2048` `4660.5 ns`, `fwht/4096` `9628.3 ns`,
  `score_ip_encoded/d1536_b4` `2405.7 ns`, `score_ip_codes_lite/d1536_b4` `2147.1 ns`
- auto (`avx2+fma`): `fwht/2048` `4272.2 ns`, `fwht/4096` `8733.7 ns`,
  `score_ip_encoded/d1536_b4` `1973.0 ns`, `score_ip_codes_lite/d1536_b4` `2230.8 ns`

Experiment log:
- kept: aligned 3-bit decode specialization for `score_ip_encoded`, `score_ip_from_parts`, and
  `score_ip_codes_lite`; this is the first follow-up that materially reduced the remaining scalar
  decode overhead on the frozen scorer APIs
- observed from the matched `5000`-iteration before/after runs: `score_ip_encoded` improved by
  about `4.7x` in scalar mode and `4.2x` in auto mode; `score_ip_codes_lite` improved by about
  `6.0x`, with the auto number matching scalar because that scorer still intentionally dispatches
  to scalar on this host
- kept: generic fallback for non-3-bit widths; no planner, scan, or graph-search files changed
- discarded earlier on this branch: pointer-style FWHT rewrite in `src/quant/hadamard.rs`; it did
  not produce a stable throughput win on this host
- discarded earlier on this branch: `codebook_products` precompute table for
  `score_ip_codes_lite`; the win was too small/noisy to justify the added cached state
- retained from the previous checkpoint: `score_ip_codes_lite` stays on scalar dispatch for now
  because the earlier SIMD version was slower than scalar on this host

Please review:
- whether the aligned 3-bit helper is narrow enough and obviously equivalent to the existing
  packer/unpacker contract for the common 4-bit quantizer case
- whether the scalar and AVX2 loops still handle tail dimensions correctly after chunking by 8
- whether recording discarded B1 experiments in the request packet is sufficient, or whether you
  want a separate experiment-log file under `review/` for later checkpoints
