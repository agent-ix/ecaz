# Review Request: C1 ADR-030 V2 Grouped PQ Feasibility Spike

## Context

Packet `310` defined the ADR-030 v2 direction:

- current-format grouped reinterpretation is retired
- v2 should use a true grouped search code
- the smallest next step is an offline feasibility study on transformed data

## Problem

Before touching metadata, tuple layout, or runtime search, we need one narrow answer:

1. does true grouped `PQ4` on transformed tqvector data recover enough ranking quality to justify
   the redesign
2. and is the remaining error mostly in the grouped encoding itself or in LUT quantization

If the answer is "no", ADR-030 should stop before broad index-v2 implementation work.

## Planned Slice

Extend `src/bin/approx_score_study.rs` with a true grouped-code path that:

1. trains per-subvector `PQ4` codebooks on `SRHT`-transformed vectors
2. encodes corpus vectors into packed grouped codes
3. prepares grouped query LUTs from transformed queries
4. scores with both `f32` and quantized-LUT variants
5. compares against fp32 truth on the existing overlap/capture metrics

This slice stays offline and does not touch ordered-scan runtime.

## Implementation Checkpoint

The feasibility spike is now implemented in `src/bin/approx_score_study.rs`.

What landed:

1. new study modes:
   - `--study-mode grouped-pq-f32`
   - `--study-mode grouped-pq-u8`
2. new grouped-PQ training controls:
   - `--train-size`
   - `--kmeans-iters`
3. true grouped `PQ4` training on `SRHT`-rotated data:
   - per-subvector `k=16` codebooks
   - packed two-nibbles-per-byte grouped search codes
   - query-time grouped LUT build in both `f32` and row-quantized `u8` forms
4. narrow unit coverage for the packed grouped-code path and `u8` vs `f32` grouped scoring

What did **not** land:

- no runtime scan integration
- no metadata-page or tuple-layout changes
- no `OPQ` branch yet

## Validation

Green code checkpoint:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused study validation also ran:

- `cargo run --bin approx_score_study -- --study-mode grouped-pq-f32 --corpus-size 256 --query-count 3 --bench-iters 1 --group-size 16 --train-size 128 --kmeans-iters 4`
- `cargo run --release --bin approx_score_study -- --study-mode grouped-pq-f32 --group-size 16 --train-size 4096 --kmeans-iters 15 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`
- `cargo run --release --bin approx_score_study -- --study-mode grouped-pq-u8 --group-size 16 --train-size 4096 --kmeans-iters 15 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

## Measurements

### Focused synthetic smoke

All known focused smoke results:

- `study-mode=grouped-pq-f32`
- `corpus=256`
- `queries=3`
- `group_size=16`
- `train_size=128`
- `kmeans_iters=4`
- `spearman_rho mean=0.4608 min=0.4049`
- `pearson_r mean=0.4801 min=0.4443`
- `top10_overlap mean=0.2000`
- `exact_top10_captured_by_approx_top20 mean=0.4000`
- `exact_top10_captured_by_approx_top50 mean=0.6667`
- `exact_top10_captured_by_approx_top100 mean=0.8333`
- `exact_top10_captured_by_approx_top200 mean=0.9667`
- `group_count=96`
- `grouped_code_bytes=48`
- `microbench exact_ns_per_score=22299.3`
- `microbench grouped_pq_f32_ns_per_score=2078.9`
- `microbench grouped_pq_u8_ns_per_score=2927.3`

This smoke run was only a sanity check that the new grouped-code path behaved plausibly before the
real-corpus release reads.

### Real-corpus release results

All known real-corpus release runs for this attempt:

- `study-mode=grouped-pq-f32`
  - `group_size=16`
  - `train_size=4096`
  - `kmeans_iters=15`
  - `spearman_rho mean=0.8859 min=0.7761`
  - `pearson_r mean=0.8898 min=0.7873`
  - `top10_overlap mean=0.7100`
  - `exact_top10_captured_by_approx_top20 mean=0.8250`
  - `exact_top10_captured_by_approx_top50 mean=0.8950`
  - `exact_top10_captured_by_approx_top100 mean=0.9250`
  - `exact_top10_captured_by_approx_top200 mean=0.9600`
  - `exact_top10_captured_by_approx_top500 mean=0.9850`
  - `exact_top10_captured_by_approx_top1000 mean=0.9950`
  - `group_count=96`
  - `grouped_code_bytes=48`
  - `microbench exact_ns_per_score=1793.0`
  - `microbench grouped_pq_f32_ns_per_score=115.6`
  - `microbench grouped_pq_u8_ns_per_score=157.4`
  - `grouped_pq_f32_speedup=15.51x`
  - `grouped_pq_u8_speedup=11.39x`
- `study-mode=grouped-pq-u8`
  - `group_size=16`
  - `train_size=4096`
  - `kmeans_iters=15`
  - `spearman_rho mean=0.8859 min=0.7760`
  - `pearson_r mean=0.8898 min=0.7872`
  - `top10_overlap mean=0.7150`
  - `exact_top10_captured_by_approx_top20 mean=0.8250`
  - `exact_top10_captured_by_approx_top50 mean=0.8950`
  - `exact_top10_captured_by_approx_top100 mean=0.9250`
  - `exact_top10_captured_by_approx_top200 mean=0.9600`
  - `exact_top10_captured_by_approx_top500 mean=0.9850`
  - `exact_top10_captured_by_approx_top1000 mean=0.9950`
  - `group_count=96`
  - `grouped_code_bytes=48`
  - `microbench exact_ns_per_score=1834.8`
  - `microbench grouped_pq_f32_ns_per_score=163.0`
  - `microbench grouped_pq_u8_ns_per_score=191.2`
  - `grouped_pq_f32_speedup=11.25x`
  - `grouped_pq_u8_speedup=9.60x`

### Comparison point versus packet `280`

Packet `280` on the current-format grouped reinterpretation at `group_size=16` reported:

- `spearman_rho mean=0.7024`
- `top10_overlap mean=0.6500`
- `exact_top10_captured_by_approx_top100 mean=0.9000`
- `grouped_f32_ns_per_score=758.3`

This attempt's true grouped-code SRHT result at the same `group_size=16` improved to:

- `spearman_rho mean=0.8859`
- `top10_overlap mean=0.7100-0.7150`
- `exact_top10_captured_by_approx_top100 mean=0.9250`
- `grouped_pq_f32_ns_per_score=115.6`

So the main packet-`280` failure really was the current-format reinterpretation, not the grouped
search-code idea itself.

## Outcome

Kept as a promising feasibility result.

The result is **not** yet enough to claim that ADR-030 v2 clears the long-horizon target on its
own. But it is enough to justify continuing the v2 lane.

Key reads:

1. true grouped `PQ4` on transformed data is materially stronger than the rejected current-format
   grouped reinterpretation from packet `280`
2. `u8` LUT quantization is again basically a non-issue here; the `u8` run tracks the `f32` run
   almost exactly
3. the grouped search code is still weaker than packet `279` / ADR-031 binary sign on raw
   correlation (`0.8859` vs about `0.9320`), so the right architecture is still the composed one:
   `binary prefilter + grouped FastScan + tiny rerank`
4. the new grouped code is compact (`48B`) and much faster than the exact scorer in the study
   harness (`~9.6x-15.5x` depending on the microbench readout)

So ADR-030 v2 now has a real green light:

- keep the grouped search-code redesign alive
- do not treat it as a single-stage replacement yet
- design the next slice around the composed pipeline

## Next Step

The next narrow ADR-030 slice should move from feasibility into format contract:

1. add explicit metadata-page versioning
2. define the v2 transform and payload descriptors
3. sketch the hot tuple / cold rerank layout
4. only then choose the smallest builder/runtime seam that emits and consumes the grouped search
   code
