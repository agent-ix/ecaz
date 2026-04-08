# Review Request: AVX2 FWHT 64-Lane Bootstrap

Commit: `d276990`

Scope:
- `src/quant/hadamard.rs`

Summary:
- extend the AVX2 FWHT bootstrap to 64 elements by reusing the 32-point helper on both halves of a
  64-lane chunk, then applying the `width = 32` butterfly in registers
- start the existing wider-stage loop at `width = 64` when the input length allows it
- add a focused x86_64 test that the 64-lane helper matches scalar output when AVX2 is available

Before/after AVX2 FWHT snapshot on this machine (same harness at `5000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`a631d45`, recorded in `review/10104`):
  `fwht/2048` `1235.7 ns`, `fwht/4096` `2808.5 ns`
- after (`d276990`, rerun for this packet):
  `fwht/2048` `1201.6 ns`, `fwht/4096` `2698.2 ns`

Current FWHT throughput snapshot on this machine (`20000` iterations, `warmup_iterations=256`):
- auto (`avx2+fma`): `fwht/2048` `1228.2 ns`, `fwht/4096` `2648.2 ns`
- scalar: `fwht/2048` `3756.9 ns`, `fwht/4096` `7770.3 ns`

Experiment log:
- kept: a 64-point AVX2 bootstrap that folds the `width = 32` stage into the same first-pass
  register work as the earlier `1/2/4/8/16` bootstrap chain
- observed from matched `5000`-iteration runs: `fwht/2048` improved by about `2.8%` and
  `fwht/4096` by about `3.9%` versus `10104`
- observed from the warmed `20000`-iteration runs on this host: FWHT is now about `3.06x` scalar
  at `2048` and about `2.93x` at `4096`
- unchanged from the prior caveat: this is better than `10104`, but still not stable enough on
  this machine to claim `BC-008` closed for all representative sizes
- unchanged by design: scalar FWHT, NEON FWHT, encoded scoring, and lite scoring logic are
  untouched

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether extending the bootstrap boundary to 64 elements is still a reasonable complexity tradeoff
- whether the helper-level coverage is sufficient for this fourth FWHT specialization step
- whether the packet describes the local throughput improvement clearly without overstating `BC-008`
