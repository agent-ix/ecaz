# Review Request: AVX2 4-Accumulator Unroll

Commit: `ce9e548`

Scope:
- `src/quant/prod.rs`

Summary:
- keep scalar, NEON, and non-3-bit AVX2 paths unchanged
- widen both AVX2 3-bit score loops from 2 accumulators (16-dim unroll) to 4 accumulators
  (32-dim unroll)
- `score_ip_mse_codes_avx2`: `mse_acc0..1` → `mse_acc0..3`
- `score_ip_from_split_parts_avx2`: `mse_acc0..1, qjl_acc0..1` → `mse_acc0..3, qjl_acc0..3`
- fold with balanced tree: `add(add(acc0, acc1), add(acc2, acc3))` before horizontal reduction
- fall through to 8-dim tail (same as before) for residual dimensions

What changed technically:
- the outer loop now processes 4 × 8 = 32 dimensions per iteration, issuing 4 independent
  decode + permute + accumulate chains
- the lite path (`score_ip_mse_codes_avx2`) issues 8 permutes per 32-dim iteration (was 4 per
  16-dim), with each pair of permute+multiply feeding a distinct accumulator
- the encoded path (`score_ip_from_split_parts_avx2`) issues 4 permutes + 4 FMAs for MSE and 4
  QJL FMAs per 32-dim iteration, interleaved across 8 independent accumulators
- register pressure: the lite path uses 7 YMM registers for accumulators+constants (codebook,
  shifts, mask, acc0-3), leaving 9 for temporaries; the encoded path uses 11 (8 accumulators +
  codebook + shifts + mask), leaving 5 for temporaries — tight but workable

Why 4 accumulators help the lite path more:
- the lite loop body per 8-dim chunk is: 2 decodes + 2 permutes + 1 multiply + 1 add
- `permutevar8x32_ps` has 3 cycle latency on port 5 — with 4 permutes per 16-dim iteration
  (2-accumulator), the accumulator chain was a significant fraction of the critical path
- widening to 4 accumulators gives the OOO engine more independent work to overlap permute
  latency from successive chunks
- the encoded path already had diverse per-iteration work (permute + FMA + memory loads for
  rotated/sq/sign_lanes), giving the OOO engine enough to hide the 2-accumulator chain —
  widening provides no additional benefit

Rejected alternative — QJL sign expansion via SIMD bit manipulation:
- tried replacing `qjl_sign_lanes` LUT lookup with inline AVX2 bit-to-sign expansion
  (broadcast + AND + cmpeq + AND → XOR sign flip)
- this eliminated the LUT load but added integer ALU pressure on ports 0/1/5 that competed
  with the decode and permute operations
- result: ~6% regression on `score_ip_encoded` — the 8KB LUT fits in L1 and the load ports
  (2/3) were underutilized relative to compute ports, so the LUT was actually cheaper
- reverted; recorded here as a dead end

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- baseline with 2 accumulators (commit `cc3c620`):
  - `score_ip_encoded/d1536_b4` `517.6 ns`
  - `score_ip_codes_lite/d1536_b4` `673.8 ns`
- after 4 accumulators (first run):
  - `score_ip_encoded/d1536_b4` `512.9 ns`
  - `score_ip_codes_lite/d1536_b4` `644.6 ns`
- confirmatory rerun:
  - `score_ip_encoded/d1536_b4` `517.3 ns`
  - `score_ip_codes_lite/d1536_b4` `650.9 ns`

Observed deltas (confirmatory run vs baseline):
- `score_ip_encoded/d1536_b4`: flat (within measurement noise)
- `score_ip_codes_lite/d1536_b4`: about `3.4%` faster

Cumulative deltas from original baseline (pre `f38a4e3`):
- `score_ip_encoded/d1536_b4`: about `34.8%` faster (`793.9` → `517.3 ns`)
- `score_ip_codes_lite/d1536_b4`: about `52.7%` faster (`1376.0` → `650.9 ns`)

Validation:
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`

Please review:
- whether 4 accumulators is the right stopping point for unroll depth, or whether the
  diminishing returns indicate the bottleneck has shifted entirely to instruction throughput
- whether the non-3-bit `else` branch should also be widened (currently still single-accumulator
  with scalar index extraction per lane)
- what the next scoring bottleneck is — permute throughput on the lite path, or load bandwidth
  on the encoded path
