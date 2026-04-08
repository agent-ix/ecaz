# Review Request: AVX2 FWHT 1024 Bench and 2048 Tile Tuning

Commit: `ecc7c19`

Scope:
- `src/quant/hadamard.rs`
- `src/bin/simd_bench.rs`

Summary:
- add `fwht/1024` to the SIMD benchmark harness so the ADR-020 speed-candidate dimension is
  measured directly alongside `2048` and `4096`
- retune the exact-size AVX2 tile chooser for `2048`:
  - previous tiled checkpoint used `2048 -> 1024`
  - this checkpoint uses `2048 -> 256`
- leave the current `1024` runtime path on the existing `512`-tile schedule after two tighter-tile
  experiments failed to produce a stable win
- leave `4096+` on the existing `512`-tile path

Matched AVX2 FWHT heuristic comparison on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`, with `fwht/1024` included first in both runs):
- before (previous `2048 -> 1024` heuristic, measured locally before this commit):
  `fwht/1024` `530.1 ns`, `fwht/2048` `1176.1 ns`, `fwht/4096` `2603.5 ns`
- after (`ecc7c19`, first long run after the `2048 -> 256` change):
  `fwht/1024` `543.8 ns`, `fwht/2048` `1118.3 ns`, `fwht/4096` `2581.8 ns`

Current snapshot on this machine:
- scalar (`ecc7c19`): `fwht/1024` `2306.5 ns`, `fwht/2048` `4637.8 ns`, `fwht/4096` `9521.3 ns`
- auto reruns on `ecc7c19`:
  - run A: `fwht/1024` `543.8 ns`, `fwht/2048` `1118.3 ns`, `fwht/4096` `2581.8 ns`
  - run B: `fwht/1024` `603.0 ns`, `fwht/2048` `1141.7 ns`, `fwht/4096` `2591.1 ns`
  - run C: `fwht/1024` `525.1 ns`, `fwht/2048` `1114.2 ns`, `fwht/4096` `2734.7 ns`

Experiment log:
- previous tiled FWHT checkpoint already recorded the discarded recursive split/combine attempt in
  `review/10107`
- tried and kept here: tighter exact-size tiling for `2048` only
- observed from the matched heuristic comparison: `fwht/2048` improved by about `4.9%`
  (`1176.1 ns -> 1118.3 ns`) on the first long run with the same benchmark order
- observed from the extra reruns: `fwht/2048` stayed in the `1114-1142 ns` band on this host,
  which is still below the earlier `1176.1 ns` run
- unchanged by design: `1024` and `4096+` do not use the new `2048 -> 256` branch, so movement in
  those sizes across reruns should be treated as measurement noise rather than an intended effect
- tried and discarded for exact-size `1024`:
  - `1024 -> 256`: first run `510.6 ns`, immediate rerun `530.3 ns`; not a stable win over the
    earlier `530.1 ns` band
  - `1024 -> 128`: `538.6 ns`; clear regression
- final choice: keep the new `fwht/1024` benchmark, but keep the `1024` runtime heuristic on the
  earlier `512`-tile schedule until a stronger signal appears

Validation:
- `cargo test fwht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `TQVECTOR_SIMD=scalar cargo run --bin simd_bench --release --no-default-features --features pg17 -- 20000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether tightening the exact-size `2048` tile from `1024` to `256` is a reasonable
  benchmark-driven refinement to the tiled FWHT schedule
- whether keeping `fwht/1024` in the benchmark harness is the right boundary now that ADR-020
  treats `1024` as the speed candidate
- whether the packet records the discarded `1024` tile experiments clearly enough to avoid
  retracing the same unstable path later
