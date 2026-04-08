# Review Request: AVX2 FWHT 32-Lane Bootstrap

Commit: `a631d45`

Scope:
- `src/quant/hadamard.rs`

Summary:
- extend the AVX2 FWHT bootstrap to 32 elements by reusing the 16-point helper on both halves of a
  32-lane chunk, then applying the `width = 16` butterfly in registers
- start the existing wider-stage loop at `width = 32` when the input length allows it
- add a focused x86_64 test that the 32-lane helper matches scalar output when AVX2 is available

Before/after AVX2 FWHT snapshot on this machine (same harness at `5000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`d836d0e`, recorded in `review/10103`):
  `fwht/2048` `1386.9 ns`, `fwht/4096` `3068.6 ns`
- after (`a631d45`, rerun for this packet):
  `fwht/2048` `1235.7 ns`, `fwht/4096` `2808.5 ns`

Current FWHT throughput snapshot on this machine (`20000` iterations, `warmup_iterations=256`):
- auto (`avx2+fma`): `fwht/2048` `1236.5 ns`, `fwht/4096` `2786.2 ns`
- scalar: `fwht/2048` `3373.9 ns`, `fwht/4096` `7129.6 ns`

Experiment log:
- kept: a 32-point AVX2 bootstrap that folds the `width = 16` stage into the same first-pass
  register work as the earlier `1/2/4/8` bootstrap chain
- observed from matched `5000`-iteration runs: `fwht/2048` improved by about `10.9%` and
  `fwht/4096` by about `8.5%` versus `10103`
- observed from the warmed `20000`-iteration runs on this host: FWHT is now about `2.73x` scalar
  at `2048` and about `2.56x` at `4096`
- unchanged from the prior caveat: this is better than `10103`, but still not stable enough on
  this machine to claim `BC-008` closed
- unchanged by design: scalar FWHT, NEON FWHT, and the quantizer scoring paths are untouched

Please review:
- whether extending the bootstrap boundary to 32 elements is still a reasonable complexity tradeoff
- whether the helper-level coverage is sufficient for this third FWHT specialization step
- whether the packet is still clear about FWHT progress without overstating the local throughput
  result
