# Review Request: Task 124 / 029 TQ Query Prep LUT16

## Summary

This slice addresses required Task 124 lever 2: **per-query LUT / query-prep cost**.

Change:

- Added a 16-centroid specialization to `build_prepared_query_lut`, matching the existing 8-centroid specialization.
- Added an ignored Task 124 TQ-internal profiler test that splits no-QJL 4-bit query prep into:
  - full `prepare_ip_query_lut_no_qjl_4bit`;
  - `srht_padded`;
  - `build_prepared_query_lut_16`.

This is not an end-to-end latency claim and does not compare against f32/source scoring. It measures the TurboQuant query-prep compute path directly.

## TQ-Internal Result

Primary comparable hot-run delta from `artifacts/manifest.md`:

- Full no-QJL LUT query prep: `4484.6 ns -> 4170.9 ns` per query prep (`-7.0%`).
- LUT build component: `1888.6 ns -> 1535.1 ns` per LUT build (`-18.7%`).
- SRHT component was effectively unchanged/noise: `1878.6 ns -> 1901.3 ns`.

The release-rebuild-adjacent profile logs were noisy, so the request cites the immediate hot reruns as the primary before/after pair and keeps all raw profile logs in `artifacts/`.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test --release --lib --features bench quant::prod::tests::quantizer_1536_4bit_supports_explicit_lut_query_prep -- --nocapture`
- `cargo test --release --lib --features bench quant::prod::tests::explicit_lut_no_qjl_4bit_matches_direct_scoring -- --nocapture`
- `ECAZ_TQ_QUERY_PREP_PROFILE_ITERS=2000 ECAZ_TQ_QUERY_PREP_PROFILE_LOG=reviews/task-124/029-tq-query-prep-lut16/artifacts/query-prep-lut16-final-hot-rerun.log cargo test --release --lib --features bench task124_profile_no_qjl_lut_query_prep -- --ignored --nocapture`

## Scope

Kept:

- 16-centroid no-QJL 4-bit LUT construction specialization.
- TQ query-prep profiler harness for repeatable Task 124 internal timing.

Not attempted in this slice:

- Batch/flush width sweep.
- Dimension/subspace reduction.
- TQ2 SIMD kernel.
- QJL scoring speed.
- Payload prefetch/pipelining.

Task 124 remains open; this packet covers only one of the required seven levers.

