# Review Request: AVX2 FWHT 1024 Two-Level Exact Size

Commit: `a04592f`

Scope:
- `src/quant/hadamard.rs`

Summary:
- keep the current AVX2 FWHT stage-combiner and exact-size `2048` / `4096` schedules unchanged
- add one more exact-size dispatch in `fwht_in_place_avx2`:
  - `1024 -> fwht_in_place_avx2_two_level(values, 512, 256)`
- leave the generic tiled chooser in place for other transform sizes
- leave scalar FWHT, NEON FWHT, tests, and runtime dispatch boundaries unchanged

Matched AVX2 FWHT comparison on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`c6a89c4`, immediate pre-change run on this branch): `fwht/1024` `512.9 ns`,
  `fwht/2048` `1048.1 ns`, `fwht/4096` `2453.9 ns`
- after (`a04592f`, validated post-change run): `fwht/1024` `489.8 ns`, `fwht/2048` `1054.2 ns`,
  `fwht/4096` `2456.7 ns`

Current snapshot on this machine:
- auto reruns on `a04592f`:
  - run A: `fwht/1024` `489.3 ns`, `fwht/2048` `1052.7 ns`, `fwht/4096` `2428.4 ns`
  - run B: `fwht/1024` `464.9 ns`, `fwht/2048` `1057.9 ns`, `fwht/4096` `2416.5 ns`
  - run C: `fwht/1024` `489.8 ns`, `fwht/2048` `1054.2 ns`, `fwht/4096` `2456.7 ns`
- scalar snapshot for context (`a04592f`, same harness): `fwht/1024` `2345.6 ns`,
  `fwht/2048` `4673.2 ns`, `fwht/4096` `9524.8 ns`

Experiment log:
- `review/10109` previously tried tighter `1024` tiling and did not keep it:
  - `1024 -> 256` was not a stable win on the older stage-combiner path
  - `1024 -> 128` regressed
- `review/10111` then reduced stage-walk overhead materially
- tried and kept here: reopen `1024` exact-size tuning on top of the faster stage combiner, but
  only with the narrow `512/256` two-level schedule
- observed from the matched before/final run:
  - `fwht/1024` improved from `512.9 ns` to `489.8 ns`, about `4.5%`
  - `fwht/2048` and `fwht/4096` stayed in the same band; any movement there should be treated as
    noise because this checkpoint only changes the `1024` dispatch
- observed from the extra reruns:
  - `fwht/1024` stayed below the pre-change `512.9 ns` baseline in all three measured runs
  - the widened run-to-run variance noted in `review/10109` did not erase the win after the
    `review/10111` stage-combiner change

Validation:
- `cargo test fwht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `TQVECTOR_SIMD=scalar cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether reopening `1024` exact-size specialization after the stage-combiner improvement is a
  reasonable benchmark-driven follow-on to `review/10111`
- whether `1024 -> (outer 512, inner 256)` is the right narrow specialization boundary now that it
  shows a repeatable win on this host
- whether the packet explains clearly why the earlier discarded `1024` tuning in `review/10109`
  does not make this slice redundant
