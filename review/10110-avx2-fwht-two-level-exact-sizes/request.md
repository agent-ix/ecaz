# Review Request: AVX2 FWHT Two-Level Exact-Size Schedules

Commit: `ae9f559`

Scope:
- `src/quant/hadamard.rs`

Summary:
- specialize exact-size AVX2 FWHT scheduling for `2048` and `4096` above the existing tiled path
- add a two-level AVX2 helper that:
  - fully transforms `256`-lane inner tiles with the existing `8/16/32/64`-lane AVX2 leaf chain
  - combines those inner tiles inside a larger exact-size outer tile
  - only then runs the final whole-array combine stage
- keep the exact-size schedules narrow and benchmark-driven:
  - `2048`: `outer_tile_width=1024`, `inner_tile_width=256`
  - `4096`: `outer_tile_width=2048`, `inner_tile_width=256`
- extend direct x86_64 correctness coverage so the AVX2 tiled path is exercised at exact sizes
  `128/256/512/1024/2048/4096`
- leave scalar FWHT, NEON FWHT, and runtime dispatch boundaries unchanged

Matched AVX2 FWHT comparison on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`cf829be`, rerun from a detached baseline worktree): `fwht/1024` `521.7 ns`,
  `fwht/2048` `1112.3 ns`, `fwht/4096` `2603.0 ns`
- after (`ae9f559`): `fwht/1024` `523.6 ns`, `fwht/2048` `1085.4 ns`, `fwht/4096` `2540.6 ns`

Current scalar vs auto snapshot on this machine (`ae9f559`, same harness at `20000` iterations):
- auto (`avx2+fma`): `fwht/1024` `523.6 ns`, `fwht/2048` `1085.4 ns`, `fwht/4096` `2540.6 ns`
- scalar: `fwht/1024` `2267.4 ns`, `fwht/2048` `4558.6 ns`, `fwht/4096` `9473.4 ns`

Experiment log:
- discarded recursive split/combine AVX2 FWHT is already recorded in `review/10107`
- generic tiled outer-stage schedule is recorded in `review/10108`
- exact-size `2048 -> 256` tuning and discarded `1024` retuning are recorded in `review/10109`
- tried and kept here: two-level exact-size schedules that keep one more combine stage inside a
  larger local tile before returning to the full-array stage
- observed from the same-host A/B run:
  - `fwht/1024` moved from `521.7 ns` to `523.6 ns` on this run, about `0.4%` slower
  - `fwht/2048` improved from `1112.3 ns` to `1085.4 ns`, about `2.4%` faster
  - `fwht/4096` improved from `2603.0 ns` to `2540.6 ns`, about `2.4%` faster
- rationale for keeping the slice:
  - ADR-020 makes `2048` the padded FWHT/query-prep boundary shared by `1536` and `2048`
  - this checkpoint improves that boundary without widening the dispatch surface or changing
    non-x86 paths
- unchanged by design:
  - `1024` still uses the generic tiled AVX2 path
  - the AVX2 leaf math is still the existing `8/16/32/64`-lane bootstrap chain

Validation:
- `cargo test fwht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `TQVECTOR_SIMD=scalar cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether specializing exact-size `2048` and `4096` with a two-level AVX2 schedule is a
  reasonable complexity/perf tradeoff above the generic tiled path
- whether keeping a near-flat `1024` result is acceptable given the measured `2048` and `4096`
  wins and ADR-020's query-prep focus
- whether the packet clearly separates this kept two-level schedule from the earlier discarded
  recursive schedule and prior tile-width tuning
