# Review Request: AVX2 FWHT Stage Pointer Walk

Commit: `7193cbb`

Scope:
- `src/quant/hadamard.rs`

Summary:
- keep the current AVX2 FWHT scheduling unchanged, including:
  - the existing `8/16/32/64`-lane AVX2 leaf helpers
  - the generic tiled path
  - the exact-size `2048` and `4096` two-level schedules from `review/10110`
- tighten the AVX2 stage combiner itself:
  - split the stage loop into a per-width helper
  - replace `chunks_exact_mut(...).split_at_mut(...)` traversal with a direct pointer walk over
    `(left, right)` stage pairs
  - keep a scalar fallback only for widths below one AVX2 register
- leave scalar FWHT, NEON FWHT, and runtime dispatch boundaries unchanged

Matched AVX2 FWHT comparison on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`e2f3e5a`, immediate pre-change run on this branch): `fwht/1024` `546.2 ns`,
  `fwht/2048` `1118.7 ns`, `fwht/4096` `2563.2 ns`
- after (`7193cbb`, validated post-change run): `fwht/1024` `523.7 ns`, `fwht/2048` `1088.7 ns`,
  `fwht/4096` `2414.0 ns`

Current snapshot on this machine:
- auto reruns on `7193cbb`:
  - run A: `fwht/1024` `518.1 ns`, `fwht/2048` `1047.0 ns`, `fwht/4096` `2430.4 ns`
  - run B: `fwht/1024` `513.2 ns`, `fwht/2048` `1073.7 ns`, `fwht/4096` `2393.9 ns`
  - run C: `fwht/1024` `523.7 ns`, `fwht/2048` `1088.7 ns`, `fwht/4096` `2414.0 ns`
- scalar snapshot for context (`7193cbb`, same harness): `fwht/1024` `1842.8 ns`,
  `fwht/2048` `3686.4 ns`, `fwht/4096` `7650.5 ns`

Experiment log:
- previous FWHT scheduling work is already recorded in:
  - `review/10107` for the discarded recursive split/combine attempt
  - `review/10108` for the kept tiled outer-stage schedule
  - `review/10109` for `1024` benchmarking and `2048` tile tuning
  - `review/10110` for the kept exact-size two-level `2048`/`4096` schedules
- tried and kept here: reduce AVX2 stage-walk overhead without changing the kept schedule choices
- observed from the matched before/final run:
  - `fwht/1024` improved from `546.2 ns` to `523.7 ns`, about `4.1%`
  - `fwht/2048` improved from `1118.7 ns` to `1088.7 ns`, about `2.7%`
  - `fwht/4096` improved from `2563.2 ns` to `2414.0 ns`, about `5.8%`
- observed from the extra reruns: the post-change auto path stayed below the pre-change baseline
  band for all three measured FWHT sizes on this host
- likely explanation from this slice:
  - the previous exact-size scheduling wins were not only about tile shape
  - the stage combiner's slice/chunk traversal overhead was still material at `1024+`

Validation:
- `cargo test fwht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `TQVECTOR_SIMD=scalar cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether the direct pointer walk in the AVX2 stage combiner is a reasonable safety/complexity
  tradeoff for the measured gain
- whether the packet makes it clear that this slice is a stage-walk optimization layered on top of
  the earlier kept schedule work, not a replacement for it
- whether the current tests and validation are sufficient for this lower-level AVX2 helper change
