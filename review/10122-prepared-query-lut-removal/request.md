# Review Request: Prepared-Query LUT Removal for 3-Bit Path

Commits:
- `5b6677e` Precompute prepared-query LUT rows (step A, cherry-picked from deferred branch)
- `c5373b0` Use direct codebook indexing for 3-bit scoring paths (step B)
- `cdc3881` Skip LUT build for 3-bit prepared queries (step C)

Scope:
- `src/quant/prod.rs`
- `src/bin/simd_bench.rs`
- `src/lib.rs` (test assertion update)

Summary:
- for 4-bit quantization (`bits_per_index == 3`, the default), the prepared-query LUT is no
  longer built — `prepare_ip_query` returns an empty `lut` Vec
- all scoring backends (AVX2, scalar, NEON) now use direct `codebook[index] * rotated[dim]`
  for the 3-bit case instead of `prepared.lut[dim * 8 + index]`
- non-3-bit paths are unchanged and still build/use the LUT

What changed across three commits:

Step A — LUT row-fill (cherry-picked from `coder2-b1-simd-accel-lut-fill-hold`):
- replaced nested `Vec::push` LUT builder with `build_prepared_query_lut` using pre-sized
  `vec![0.0; len]` and unrolled 8-centroid specialization
- this was the deferred experiment from review 10118, now brought forward as an improved
  baseline for non-3-bit paths

Step B — direct codebook indexing:
- scalar 3-bit path: `prepared.lut[dim * 8 + index]` → `self.codebook[index] * prepared.rotated[dim]`
- NEON path: same substitution inside the 4-lane loop body
- AVX2 and NEON tail loops: branch on `bits_per_index == 3` to use direct codebook indexing
- the AVX2 SIMD loop already used `permutevar8x32_ps(codebook, lanes)` — no change needed

Step C — skip LUT construction:
- `prepare_ip_query` now returns `Vec::new()` for the LUT when `bits_per_index == 3`
- eliminates ~12K ns of LUT build cost for the default 4-bit quantizer
- updated test assertion (`prepared_lut_len: 32 → 0`) and bench sink (`prepared.lut[0]` →
  `prepared.rotated[0]`)

Why this is safe:
- `codebook[index] * rotated[dim]` is exactly the value the LUT stored at
  `lut[dim * num_centroids + index]` — the LUT was a precomputed cache of this product
- the existing `dispatched_score_matches_scalar_on_random_inputs` test validates that all
  backends produce matching scores
- `prepared_query_score_matches_explicit_formula` now uses the same direct-codebook formula
  to compute expected values

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- original baseline (before any changes):
  - `prepare_ip_query/d1024_b4` `10050 ns`
  - `prepare_ip_query/d1536_b4` `15316 ns`
  - `prepare_ip_query/d2048_b4` `18950 ns`
- after step A (LUT row-fill only):
  - `prepare_ip_query/d1024_b4` `3857 ns`
  - `prepare_ip_query/d1536_b4` `6207 ns`
  - `prepare_ip_query/d2048_b4` `7662 ns`
- after step C (LUT build skipped entirely):
  - `prepare_ip_query/d1024_b4` `1278 ns`
  - `prepare_ip_query/d1536_b4` `2632 ns`
  - `prepare_ip_query/d2048_b4` `2727 ns`

Observed deltas (final vs original baseline):
- `d1024`: about `87.3%` faster
- `d1536`: about `82.8%` faster
- `d2048`: about `85.6%` faster

Scoring paths unchanged:
- `score_ip_encoded/d1536_b4` `522 ns` (within noise of prior `517 ns`)
- `score_ip_codes_lite/d1536_b4` `664 ns` (within noise of prior `651 ns`)

The remaining ~2.6K ns is dominated by the two SRHT rotations (`srht/d1536_td2048` benchmarks
at ~1.3K ns each).

Validation:
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`

Please review:
- whether the conditional LUT skip is the right boundary, or whether the `lut` field should be
  made `Option<Vec<f32>>` to make the empty state explicit
- whether the non-3-bit paths still need the row-fill optimization from step A, or whether
  those paths are rare enough that the original push-based builder would be fine
- whether the remaining SRHT cost (~2.6K ns) is worth targeting next, or whether scoring
  throughput is more impactful at the system level
