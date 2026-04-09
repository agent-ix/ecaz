# Review Request: Prepared Query LUT Row Fill

Commit: `4c47b26`

Scope:
- `src/quant/prod.rs`

Summary:
- replace `prepare_ip_query`'s nested `Vec::push` LUT builder with a pre-sized row-fill helper
- keep the representation unchanged: `PreparedQuery.lut` is still row-major by dimension, with
  `num_centroids` contiguous entries per rotated value
- add a direct hot-case specialization for the current `4-bit` path (`8` centroids), while keeping
  a generic row-fill fallback for other bit widths

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- baseline from the previous kept checkpoint (`10116`):
  - `prepare_ip_query/d1024_b4` `10169.7 ns`
  - `prepare_ip_query/d1536_b4` `15632.5 ns`
  - `prepare_ip_query/d2048_b4` `19911.4 ns`
- first long run after the LUT-row-fill change:
  - `prepare_ip_query/d1024_b4` `3772.7 ns`
  - `prepare_ip_query/d1536_b4` `6088.8 ns`
  - `prepare_ip_query/d2048_b4` `7781.2 ns`
- confirmatory hot rerun:
  - `prepare_ip_query/d1024_b4` `3884.2 ns`
  - `prepare_ip_query/d1536_b4` `6235.3 ns`
  - `prepare_ip_query/d2048_b4` `7680.2 ns`

Observed deltas using the confirmatory rerun versus the `10116` baseline:
- `1024`: about `61.8%` faster
- `1536`: about `60.1%` faster
- `2048`: about `61.4%` faster

Why this slice is worth keeping:
- after the FWHT/SRHT work, `prepare_ip_query` was still dominated by LUT construction overhead
- the old builder paid a branchy `push` path for every centroid slot even though the final size was
  known upfront
- this rewrite preserves semantics, adds no new SIMD/runtime-dispatch boundary, and materially cuts
  query-prep time at all three measured operating points

Validation:
- `cargo test prepared_query_score_matches_explicit_formula -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether the fixed-width `8`-centroid row fill is the right amount of specialization for the hot
  `4-bit` case
- whether keeping the generic fallback in the same helper leaves the non-`4-bit` behavior clear
  enough
- whether this is now the right query-prep baseline to build on before trying any deeper
  encode/query-prep work
