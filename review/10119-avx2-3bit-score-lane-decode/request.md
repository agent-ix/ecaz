# Review Request: AVX2 3-Bit Score Lane Decode

Commit: `f38a4e3`

Scope:
- `src/quant/prod.rs`

Summary:
- keep the packed 3-bit MSE code layout unchanged
- keep scalar and NEON behavior unchanged
- replace the AVX2 `bits_per_index == 3` score path's per-iteration scalar lane construction with
  an AVX2 lane decode from the aligned 24-bit packed word
- reuse that vector lane decode in both:
  - `score_ip_mse_codes_avx2`
  - `score_ip_from_split_parts_avx2`
- add direct AVX2 correctness coverage that the new lane decode matches the existing scalar
  `decode_eight_3bit_aligned` helper

What changed technically:
- factored the aligned 3-bit chunk load into `decode_eight_3bit_aligned_word`
- added `decode_eight_3bit_lanes_avx2(word, shifts, mask)` which:
  - broadcasts the packed 24-bit word to all lanes
  - applies per-lane right shifts `[0, 3, 6, 9, 12, 15, 18, 21]`
  - masks each lane down to one 3-bit centroid index
- this removes the hot-loop `_mm256_setr_epi32(...)` rebuild from scalar-decoded index arrays on
  every 8-lane chunk

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- baseline before this code change:
  - `score_ip_encoded/d1536_b4` `793.9 ns`
  - `score_ip_codes_lite/d1536_b4` `1376.0 ns`
- first long run after the change:
  - `score_ip_encoded/d1536_b4` `545.6 ns`
  - `score_ip_codes_lite/d1536_b4` `694.7 ns`
- confirmatory hot rerun:
  - `score_ip_encoded/d1536_b4` `554.2 ns`
  - `score_ip_codes_lite/d1536_b4` `713.0 ns`

Observed deltas from the confirmatory rerun:
- `score_ip_encoded/d1536_b4`: about `30.2%` faster
- `score_ip_codes_lite/d1536_b4`: about `48.2%` faster

Why this slice is worth keeping:
- this directly targets the current per-candidate scoring hot path rather than generic bit packing
  in the encode path
- the default `4-bit` quantizer path already had a packed-3-bit specialization, but it was still
  rebuilding AVX2 lane indices from scalar arrays inside the hot loop
- the kept change stays inside the existing AVX2 runtime-dispatch boundary and materially reduces
  score time without widening algorithm scope

Validation:
- `cargo test decode_eight_3bit -- --nocapture`
- `cargo test dispatched_score_matches_scalar_on_random_inputs -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether the AVX2 lane-decode helper is the right boundary for the aligned 3-bit score fast path
- whether reusing the same helper across encoded-score and lite-score AVX2 loops is clear enough
- whether this now shifts the next scoring bottleneck away from packed-index decode and toward later
  score accumulation work
