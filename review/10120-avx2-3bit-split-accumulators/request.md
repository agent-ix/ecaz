# Review Request: AVX2 3-Bit Split Score Accumulators

Commit: `cc3c620`

Scope:
- `src/quant/prod.rs`

Summary:
- keep packed 3-bit layout, scalar, and NEON paths unchanged
- in both AVX2 3-bit score loops (`score_ip_mse_codes_avx2` and `score_ip_from_split_parts_avx2`),
  split single accumulators into dual accumulators (`acc0` / `acc1`) and add an unrolled 16-lane
  outer loop that processes two 8-lane chunks per iteration
- fold `acc0 + acc1` at the tail before horizontal reduction

What changed technically:
- `score_ip_mse_codes_avx2` (3-bit path): `mse_acc` → `mse_acc0` + `mse_acc1`; outer loop now
  decodes two packed words and issues two independent `_mm256_mul_ps` + `_mm256_add_ps` chains per
  iteration before falling through to the existing 8-lane tail
- `score_ip_from_split_parts_avx2` (3-bit path): `mse_acc` + `qjl_acc` → four accumulators
  (`mse_acc0`, `mse_acc1`, `qjl_acc0`, `qjl_acc1`); outer loop issues four independent
  `_mm256_fmadd_ps` chains per iteration
- the 4-bit path inside `score_ip_from_split_parts_avx2` is unchanged (single accumulators, no
  unrolled loop)
- the final `_mm256_storeu_ps` now receives `_mm256_add_ps(acc0, acc1)` instead of a single
  accumulator

Why this helps:
- FMA and permute instructions have multi-cycle latency; a single accumulator serialises iterations
  through a carried dependency chain
- two independent accumulators let the CPU overlap FMA execution from successive iterations,
  improving throughput on out-of-order cores

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- baseline before `f38a4e3` (pre lane-decode):
  - `score_ip_encoded/d1536_b4` `793.9 ns`
  - `score_ip_codes_lite/d1536_b4` `1376.0 ns`
- after `f38a4e3` (lane-decode, single accumulators):
  - `score_ip_encoded/d1536_b4` `554.2 ns`
  - `score_ip_codes_lite/d1536_b4` `713.0 ns`
- after `cc3c620` (split accumulators, this change):
  - `score_ip_encoded/d1536_b4` `513.9 ns`
  - `score_ip_codes_lite/d1536_b4` `702.5 ns`

Observed deltas from split accumulators vs single accumulators:
- `score_ip_encoded/d1536_b4`: about `7.3%` faster
- `score_ip_codes_lite/d1536_b4`: about `1.5%` faster

Cumulative deltas from original baseline:
- `score_ip_encoded/d1536_b4`: about `35.3%` faster
- `score_ip_codes_lite/d1536_b4`: about `48.9%` faster

Why the gain is larger on the encoded path:
- `score_ip_mse_codes_avx2` has a tighter loop body (permute + mul + add, no FMA, no extra memory
  loads), so the carried dependency on a single accumulator is a larger fraction of total loop cost
- `score_ip_from_split_parts_avx2` already has more independent work per iteration (two FMAs plus
  memory loads for `rotated` and `sq`), giving the CPU more to hide latency behind even with a
  single accumulator

Validation:
- `cargo test decode_eight_3bit -- --nocapture`
- `cargo test dispatched_score_matches_scalar_on_random_inputs -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether the two-accumulator unroll is the right depth (vs 4 accumulators or software pipelining)
- whether the asymmetric gain (encoded path benefits more than lite path) is consistent with the
  expected dependency structure
- whether this now shifts the next bottleneck toward the permute/gather or toward memory bandwidth
  on the rotated/sq loads
