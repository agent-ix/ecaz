# Task 124 / 029 TQ Query Prep LUT16 Manifest

- Head SHA: `3306ccf7f704737e389595dcd6ab1993d20fb616`
- Task bucket: `reviews/task-124/029-tq-query-prep-lut16`
- Lane: TurboQuant no-QJL 4-bit per-query LUT/query-prep microprofile
- Fixture: `ProdQuantizer::new(1536, 4, 42)`, 1536D unit query, NEON backend
- Storage / rerank mode: not applicable; this is TQ-internal query-prep compute, not end-to-end IVF latency
- Measurement command:
  - `ECAZ_TQ_QUERY_PREP_PROFILE_ITERS=2000 ECAZ_TQ_QUERY_PREP_PROFILE_LOG=<artifact> cargo test --release --lib --features bench task124_profile_no_qjl_lut_query_prep -- --ignored --nocapture`
- Validation commands:
  - `cargo fmt --check`
  - `cargo test --release --lib --features bench quant::prod::tests::quantizer_1536_4bit_supports_explicit_lut_query_prep -- --nocapture`
  - `cargo test --release --lib --features bench quant::prod::tests::explicit_lut_no_qjl_4bit_matches_direct_scoring -- --nocapture`
- Timestamp: 2026-06-30
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `query-prep-baseline.log`

Initial baseline after release rebuild. Kept for provenance but not used as the primary comparison because rebuild-adjacent runs showed high timing noise.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=14.897417ms ns_per_iter=7448.7
srht_padded total=5.052208ms ns_per_iter=2526.1
build_prepared_query_lut_16 total=6.069125ms ns_per_iter=3034.6
```

### `query-prep-baseline-hot-rerun.log`

Generic 16-centroid baseline after temporarily reverting the LUT16 specialization. Kept for provenance; rebuild-adjacent/noisy.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=16.426917ms ns_per_iter=8213.5
srht_padded total=5.624125ms ns_per_iter=2812.1
build_prepared_query_lut_16 total=7.435416ms ns_per_iter=3717.7
```

### `query-prep-baseline-hot-rerun2.log`

Primary comparable generic 16-centroid baseline. This was the immediate hot rerun after `query-prep-baseline-hot-rerun.log`.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=8.969208ms ns_per_iter=4484.6
srht_padded total=3.75725ms ns_per_iter=1878.6
build_prepared_query_lut_16 total=3.77725ms ns_per_iter=1888.6
```

### `query-prep-lut16-unrolled.log`

First candidate LUT16 specialization run after release rebuild. Kept for provenance; rebuild-adjacent/noisy.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=16.092917ms ns_per_iter=8046.5
srht_padded total=6.4145ms ns_per_iter=3207.2
build_prepared_query_lut_16 total=6.57825ms ns_per_iter=3289.1
```

### `query-prep-lut16-unrolled-rerun.log`

Immediate hot rerun of the first LUT16 specialization candidate. This showed the candidate was promising enough to rerun a comparable hot baseline.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=8.31825ms ns_per_iter=4159.1
srht_padded total=3.764292ms ns_per_iter=1882.1
build_prepared_query_lut_16 total=3.059959ms ns_per_iter=1530.0
```

### `query-prep-lut16-final.log`

Final post-change run after restoring the LUT16 specialization. Kept for provenance; rebuild-adjacent/noisy.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=14.451666ms ns_per_iter=7225.8
srht_padded total=5.68175ms ns_per_iter=2840.9
build_prepared_query_lut_16 total=5.071791ms ns_per_iter=2535.9
```

### `query-prep-lut16-final-hot-rerun.log`

Primary comparable post-change measurement. This was the immediate hot rerun after `query-prep-lut16-final.log`.

Key lines:

```text
task124_query_prep_profile backend=neon dim=1536 iterations=2000
prepare_ip_query_lut_no_qjl_4bit total=8.341792ms ns_per_iter=4170.9
srht_padded total=3.802583ms ns_per_iter=1901.3
build_prepared_query_lut_16 total=3.07025ms ns_per_iter=1535.1
```

## Primary Delta

Using the immediate hot reruns for a like-for-like comparison:

- Full TQ query prep: `4484.6 ns -> 4170.9 ns` per query prep (`-313.7 ns`, `-7.0%`).
- TQ LUT build component: `1888.6 ns -> 1535.1 ns` per LUT build (`-353.5 ns`, `-18.7%`).
- SRHT component: `1878.6 ns -> 1901.3 ns`; not the target of this slice and effectively noise.

