# Review Request: Orthonormal and SRHT Bench Coverage

Commit: `4a075d6`

Scope:
- `src/bin/simd_bench.rs`
- `src/quant/hadamard.rs`
- `src/quant/rotation.rs`

Summary:
- extend the SIMD benchmark harness beyond bare FWHT so it measures the paths that query prep
  actually uses:
  - `orthonormal_fwht/{1024,2048,4096}`
  - `srht/d1024_td1024`
  - `srht/d1536_td2048`
  - `srht/d2048_td2048`
- add direct runtime-equivalence coverage for:
  - `orthonormal_fwht_in_place`
  - `rotation::srht`
- keep the production hot paths behaviorally unchanged in the landed code; this checkpoint is bench
  and test surface, not a claimed runtime optimization

Current benchmark snapshot on this machine (`10000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- `fwht/1024` `503.5 ns`
- `fwht/2048` `1125.5 ns`
- `fwht/4096` `2535.1 ns`
- `orthonormal_fwht/1024` `555.1 ns`
- `orthonormal_fwht/2048` `1208.4 ns`
- `orthonormal_fwht/4096` `2766.9 ns`
- `srht/d1024_td1024` `605.3 ns`
- `srht/d1536_td2048` `1339.0 ns`
- `srht/d2048_td2048` `1271.6 ns`

Why this coverage matters:
- `src/quant/rotation.rs` and `src/quant/prod.rs` use `orthonormal_fwht_in_place` and `srht`,
  not bare `fwht_in_place`
- ADR-020 specifically frames `1024`, `1536 -> 2048`, and `2048 -> 2048` as the operating points
  that matter for encode/query-prep cost
- this harness makes those paths visible before more low-level AVX2 tuning continues

Experiment log:
- tried and reverted while preparing this slice: AVX2 vectorized post-FWHT scaling in
  `orthonormal_fwht_in_place`
  - same-harness baseline from a detached `277c6fa` worktree with only the new orthonormal bench:
    - `orthonormal_fwht/1024` `518.8 ns`
    - `orthonormal_fwht/2048` `1168.9 ns`
    - `orthonormal_fwht/4096` `2673.1 ns`
  - temporary AVX2-scaling experiment on the working branch:
    - run A: `533.6 / 1154.5 / 2658.6 ns`
    - run B: `532.2 / 1149.9 / 2721.8 ns`
  - outcome: reverted before commit; not a clean enough win

- tried and reverted while preparing this slice: AVX2 vectorized sign application in
  `rotation::srht` / `inverse_srht`
  - scalar-sign baseline on the kept harness:
    - `srht/d1024_td1024` `605.3 ns`
    - `srht/d1536_td2048` `1339.0 ns`
    - `srht/d2048_td2048` `1271.6 ns`
  - temporary AVX2-sign experiment:
    - `649.4 / 1458.9 / 1453.3 ns`
  - outcome: reverted before commit; clear regression

Validation:
- `cargo test srht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 10000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether adding explicit orthonormal and SRHT coverage to `simd_bench` is the right next step
  now that bare FWHT tuning is starting to plateau
- whether the new runtime-equivalence tests are sufficient protection for future SIMD work on these
  higher-level paths
- whether the packet records the reverted AVX2 scaling and sign-application attempts clearly enough
  to avoid retracing them later
