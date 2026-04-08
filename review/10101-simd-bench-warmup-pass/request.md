# Review Request: SIMD Bench Warmup Pass

Commit: `22b1743`

Scope:
- `src/bin/simd_bench.rs`

Summary:
- add an untimed warmup pass before each measured loop in `simd_bench`
- use `256` warmup iterations so the scorer benches touch the full 256-element payload pool before
  timing
- print `warmup_iterations` in the harness output so the recorded numbers say exactly what setup was
  used

Before/after first-run stability snapshot on this machine (`20000` iterations, auto `avx2+fma`):
- before (`16d88a7`, recorded in `review/10100`, no warmup):
  first run `score_ip_encoded/d1536_b4` `919.0 ns`, `score_ip_codes_lite/d1536_b4` `1438.1 ns`
  immediate rerun `score_ip_encoded/d1536_b4` `790.6 ns`, `score_ip_codes_lite/d1536_b4`
  `1380.2 ns`
- after (`22b1743`, rerun for this packet, `warmup_iterations=256`):
  first run `score_ip_encoded/d1536_b4` `803.2 ns`, `score_ip_codes_lite/d1536_b4` `1403.3 ns`
  immediate rerun `score_ip_encoded/d1536_b4` `791.0 ns`, `score_ip_codes_lite/d1536_b4`
  `1396.0 ns`

Current whole-harness snapshot on this machine (`5000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- `fwht/2048` `3444.7 ns`, `fwht/4096` `7128.9 ns`, `score_ip_encoded/d1536_b4` `789.9 ns`,
  `score_ip_codes_lite/d1536_b4` `1357.1 ns`

Experiment log:
- kept: a `256`-iteration untimed warmup pass, which covers the full payload/code pool used by the
  scorer benches and pulls the first `20000`-iteration sample much closer to the rerun
- observed from the `20000`-iteration pair: encoded first-run drift dropped from about `16%`
  without warmup to about `1.5%` with the kept warmup, and lite drift dropped from about `4%` to
  about `0.5%`
- discarded before this checkpoint: a `100`-iteration warmup pass; it improved the first long-run
  sample (`870.6 ns` then `800.6 ns` / `808.2 ns` on immediate reruns) but still failed to cover
  the full 256-item scorer pool, so it was replaced rather than committed
- unchanged by design: the benchmark still reports the same measured kernels and the extension code
  under test is untouched

Please review:
- whether `256` warmup iterations is the right default shape for this harness on the current
  256-item scorer pool
- whether the packet documents the rejected `100`-iteration variant clearly enough
- whether reducing first-run drift is sufficient justification for keeping this harness-only B1
  checkpoint
