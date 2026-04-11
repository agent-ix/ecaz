# Request: B1 SIMD Acceleration Merge

Commit: `4a613d7`

Summary:
- Integrates the surviving optimization commits from `origin/coder2-b1-simd-accel` onto current `main` instead of merging the stale branch wholesale.
- Brings runtime SIMD dispatch, AVX2 FWHT/scoring improvements, padded SRHT query prep, prepared-query LUT reductions, expanded SIMD equivalence coverage, and the NEON 3-bit scoring implementation onto `main`.
- Preserves current `main`'s `mse_bits()` / tiled-1536 behavior instead of reintroducing the older branch's blanket `self.bits - 1` assumptions.
- Fixes one real regression found during review: the merged branch initially skipped the prepared-query LUT for the 1536-dim no-QJL path and then tried to score through an empty LUT. The final checkpoint keys that path off `mse_bits()` and uses an explicit no-QJL MSE-only score path.

Files:
- `src/quant/simd.rs`
- `src/quant/hadamard.rs`
- `src/quant/prod.rs`
- `src/quant/rotation.rs`
- `src/quant/qjl.rs`
- `src/lib.rs`
- `src/bin/simd_bench.rs`
- `plan/tasks/08-simd.md`
- `plan/status.md`

Why this matters:
- `main` was still reporting B1 as feature-branch-only work even though the branch contained a meaningful x86_64 optimization lane that now clears its correctness gates on top of current code.
- The branch included real wins in both once-per-query prep and per-candidate scoring, but it was old enough that a direct merge would have regressed current 1536-dim tiled behavior.
- This integration keeps the wins while retaining the current `main` semantics that landed after the branch forked.

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo test --lib quant::prod::tests::dispatched_score_matches_scalar_at_production_dims -- --exact --nocapture`

Measured results:
- Current `main` vs merged branch, Criterion:
  - `quant/prepare_ip_query/d1536_b4`: about `21.88 us` -> `5.46 us`
  - `quant/score_ip_encoded/d1536_b4`: about `6.34 us` -> `1.38 us`
  - `quant/score_ip_codes_lite/d1536_b4`: about `11.86 us` -> `11.29 us`
- Merged branch, `simd_bench` auto vs `TQVECTOR_SIMD=scalar`:
  - `fwht/2048`: about `895 ns` vs `2909 ns`
  - `srht/d1536_td2048`: about `1124 ns` vs `3116 ns`
  - `prepare_ip_query/d1024_b4`: about `1064 ns` vs `3024 ns`
  - `prepare_ip_query/d2048_b4`: about `2276 ns` vs `6180 ns`

Review focus:
- Whether the integration chose the right boundary for preserving `mse_bits()` / tiled-1536 semantics instead of replaying the old branch assumptions verbatim.
- Whether the no-QJL MSE-only score path in `src/quant/prod.rs` is the right narrow repair for the regression found during merge.
- Whether the remaining aarch64 gap should stay a follow-up runtime-validation item rather than blocking the x86_64 merge that is now fully validated.
