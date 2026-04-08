# Review Request: AVX2 FWHT Tiled Outer Stages

Commit: `7bf5ed7`

Scope:
- `src/quant/hadamard.rs`

Summary:
- replace the remaining AVX2 whole-array outer-stage schedule with a tiled schedule that computes
  full FWHTs inside local `128/256/512` tiles first, then resumes whole-array stages above the tile
  boundary
- keep the existing 8/16/32/64-lane AVX2 bootstrap helpers as the only leaf implementations
- tune the tile chooser from local measurements:
  - use a `1024` tile at transform length `2048`
  - use a `512` tile at `4096` and above
  - keep `128/256/512` exact-size tiled coverage for smaller powers of two above `64`
- add direct x86_64 correctness coverage for tiled exact sizes above the 64-lane leaf
- leave scalar FWHT, NEON FWHT, and runtime dispatch boundaries unchanged

Before/after AVX2 FWHT snapshot on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`2bff69b`, rerun from a detached baseline worktree):
  `fwht/2048` `1177.5 ns`, `fwht/4096` `2640.1 ns`
- after (`7bf5ed7`):
  `fwht/2048` `1160.0 ns`, `fwht/4096` `2541.0 ns`

Current scalar vs auto snapshot on this machine (`7bf5ed7`, same harness at `20000` iterations):
- auto (`avx2+fma`): `fwht/2048` `1160.0 ns`, `fwht/4096` `2541.0 ns`
- scalar: `fwht/2048` `3666.9 ns`, `fwht/4096` `7641.3 ns`

Experiment log:
- discarded before this checkpoint: a naive recursive split/combine AVX2 FWHT regressed on this
  host and was reverted before commit; see `review/10107`
- tried and kept as the core direction: tile-local stage fusion above the 64-lane AVX2 leaf
- tried during tuning:
  - uniform `512`-tile schedule: `fwht/2048` `1162.5 ns`, `fwht/4096` `2548.8 ns`
  - uniform `1024`-tile schedule: `fwht/2048` `1151.4 ns`, `fwht/4096` `2607.5 ns`
  - kept hybrid schedule (`2048 -> 1024`, `4096+ -> 512`): `fwht/2048` `1160.0 ns`,
    `fwht/4096` `2541.0 ns`
- observed from the baseline-vs-final rerun: this slice improved `fwht/2048` by about `1.5%` and
  `fwht/4096` by about `3.8%` on this machine
- observed from the final scalar-vs-auto run: FWHT is about `3.16x` scalar at `2048` and about
  `3.01x` scalar at `4096`
- unchanged by design: the AVX2 leaf math is still the existing 8/16/32/64-lane bootstrap chain;
  this checkpoint only changes the schedule above those leaves

Validation:
- `cargo test fwht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `TQVECTOR_SIMD=scalar cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether the tiled schedule is a reasonable complexity/perf tradeoff above the current 64-lane
  AVX2 leaf
- whether the size-based tile heuristic is acceptable given the measured `2048` vs `4096`
  tradeoff on this host
- whether the packet records the kept tiled variant, the discarded recursive variant, and the
  intermediate tile-width tuning clearly enough for later SIMD work
