# Review Request: AVX2 FWHT 16-Lane Bootstrap

Commit: `d836d0e`

Scope:
- `src/quant/hadamard.rs`

Summary:
- extend the AVX2 FWHT bootstrap from 8 elements to 16 elements by reusing the 8-point helper on
  both halves of a 16-lane chunk, then applying the `width = 8` butterfly in registers
- start the existing wider-stage loop at `width = 16` when the input length allows it
- add a focused x86_64 test that the 16-lane helper matches scalar output when AVX2 is available

Before/after AVX2 FWHT snapshot on this machine (same harness at `5000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`86853f3`, recorded in `review/10102`):
  `fwht/2048` `1463.3 ns`, `fwht/4096` `3234.6 ns`
- after (`d836d0e`, rerun for this packet):
  `fwht/2048` `1386.9 ns`, `fwht/4096` `3068.6 ns`

Current FWHT throughput snapshot on this machine (`20000` iterations, `warmup_iterations=256`):
- auto (`avx2+fma`): `fwht/2048` `1386.8 ns`, `fwht/4096` `3048.5 ns`
- scalar: `fwht/2048` `3373.9 ns`, `fwht/4096` `7129.6 ns`

Experiment log:
- kept: a 16-point AVX2 bootstrap that collapses the `width = 8` stage into the same first-pass
  register work as the earlier `1/2/4` bootstrap
- observed from matched `5000`-iteration runs: `fwht/2048` improved by about `5.2%` and
  `fwht/4096` by about `5.1%` versus `10102`
- observed from the warmed `20000`-iteration runs on this host: FWHT is now about `2.43x` scalar
  at `2048` and about `2.34x` at `4096`
- important caveat: an earlier scalar run on this machine came in much slower and briefly suggested
  a `3x+` result, but reruns did not hold that baseline, so this packet does **not** treat
  `BC-008` as closed
- unchanged by design: scalar FWHT, NEON FWHT, and the quantizer scoring paths are untouched

Please review:
- whether extending the bootstrap boundary to 16 elements is still a reasonable complexity tradeoff
- whether the helper-level coverage is sufficient for this second FWHT specialization step
- whether the packet documents the unstable local throughput signal clearly enough to keep
  `BC-008` open without overstating progress
