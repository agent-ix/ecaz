# Review Request: AVX2 FWHT 8-Lane Bootstrap

Commit: `86853f3`

Scope:
- `src/quant/hadamard.rs`

Summary:
- fuse the first three FWHT butterfly stages (`width = 1, 2, 4`) into a single in-register
  8-point AVX2 transform
- keep the existing wider-stage AVX2 loop for `width >= 8`, rather than rewriting the full FWHT
  traversal
- add a focused x86_64 test that the 8-lane helper matches scalar output when AVX2 is available

Before/after AVX2 FWHT snapshot on this machine (same harness at `5000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`22b1743`, recorded in `review/10101`):
  `fwht/2048` `3444.7 ns`, `fwht/4096` `7128.9 ns`
- after (`86853f3`, rerun for this packet):
  `fwht/2048` `1463.3 ns`, `fwht/4096` `3234.6 ns`

Current throughput snapshot on this machine (`20000` iterations, `warmup_iterations=256`):
- auto (`avx2+fma`): `fwht/2048` `1444.4 ns`, `fwht/4096` `3234.0 ns`,
  `score_ip_encoded/d1536_b4` `824.3 ns`, `score_ip_codes_lite/d1536_b4` `1410.7 ns`
- scalar: `fwht/2048` `4612.8 ns`, `fwht/4096` `9631.8 ns`,
  `score_ip_encoded/d1536_b4` `2404.9 ns`, `score_ip_codes_lite/d1536_b4` `2137.5 ns`

Experiment log:
- kept: an AVX2 8-point bootstrap block that applies the `1/2/4` butterflies fully in registers,
  then hands off to the existing `width >= 8` loop
- observed from matched `5000`-iteration runs: `fwht/2048` improved by about `57.5%` and
  `fwht/4096` by about `54.6%` versus `10101`
- observed from the warmed `20000`-iteration runs: this host now shows about `3.19x` scalar-vs-AVX2
  at `fwht/2048`, which is the first local run on this branch to clear the `BC-008` target band;
  `fwht/4096` is about `2.98x`
- unchanged by design: scalar FWHT, NEON FWHT, encoded scoring, and lite scoring logic are
  untouched
- earlier discarded on this branch: a broader pointer-style FWHT rewrite recorded in `review/10038`;
  this narrower bootstrap approach is what finally produced a stable win

Please review:
- whether fusing only the `1/2/4` stages is the right specialization boundary for AVX2 FWHT
- whether the helper test is enough targeted coverage for the new 8-lane transform
- whether the local `BC-008` result is strong enough to treat FWHT as locally unblocked pending
  better-hardware confirmation
