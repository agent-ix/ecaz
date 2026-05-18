# Review Request: C1 QJL-Disabled Score Fast Path

## Context

Packet `256` materially improved the graph side of the C1 scan path. The
current best verified canonical real-`10k` `m=8` surface on `main` is now:

- `ef_search=40`: mean `69.855ms`
- `ef_search=200`: mean `124.238ms`

The remaining hot-path buckets for the representative `id=10000` probe are now
dominated by layer-0 seed work and candidate scoring:

- `ef_search=40`
  - layer-0 seed elapsed: `15.802ms`
  - candidate score elapsed: `16.217ms`
- `ef_search=200`
  - layer-0 seed elapsed: `87.147ms`
  - candidate score elapsed: `76.667ms`

On the real-corpus lane, scoring runs through the 1536-dim, 4-bit,
QJL-disabled path. `qjl_enabled(1536, 4)` is false because the tiled 1536-dim
FWHT compatibility path is active, so `score_ip_from_parts` currently falls
back to the scalar non-QJL scoring loop.

## Problem

The graph-side runtime is no longer the clean first target. The scan still
spends a large fraction of its time in quantized candidate scoring, and the
hot real-corpus path is on the exact 4-bit production configuration where the
codebook is tiny and highly regular.

That makes the scalar non-QJL score loop a likely high-leverage C1 seam.

## Planned work

1. Confirm the current non-QJL 4-bit scoring path and establish a local
   microbench baseline on the existing SIMD bench harness.
2. Add a narrow fast path for the QJL-disabled 4-bit score loop, keeping the
   existing scalar implementation as the reference path.
3. Validate correctness against the existing dispatched-vs-scalar score tests.
4. Re-run the required checkpoint gate and then measure whether the scan hot
   path and canonical real-corpus surface move materially.

## Progress

The implemented change stays deliberately narrow:

- keep `score_ip_from_split_parts_scalar` as the correctness reference path
- detect the QJL-disabled `bits = 4` lane in `score_ip_from_split_parts`
- on that lane, score directly from packed 4-bit nibbles against
  `prepared.rotated` and the tiny 16-entry codebook instead of walking the
  generic `mse_index_at + LUT` scalar loop

The required checkpoint gate is green:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Results

### SIMD bench baseline

Using the existing `src/bin/simd_bench.rs` harness on `d1536_b4`:

- `score_ip_encoded`: `68,278.4ns -> 14,279.3ns`

That is about a `79%` reduction in per-score time on the hot production
configuration.

### Representative hot-path probe

Compared to the post-greedy baseline from packet `256` on query `id=10000`:

- `ef_search=40`
  - upper-layer seed elapsed: `3.722ms -> 1.223ms`
  - layer-0 seed elapsed: `15.802ms -> 5.145ms`
  - candidate score elapsed: `16.217ms -> 3.240ms`
- `ef_search=200`
  - upper-layer seed elapsed: `3.742ms -> 1.186ms`
  - layer-0 seed elapsed: `87.147ms -> 23.698ms`
  - candidate score elapsed: `76.667ms -> 13.555ms`

The score-path speedup also shortened layer-0 traversal materially, which means
the faster scoring changed the traversal trajectory enough to reduce total scan
work, not just per-candidate cost.

### Representative SQL

On the shared canonical `tqhnsw_real_10k_m8_idx` probe:

- `ef_search=40`
  - execution time: `62.285ms -> 50.127ms`
  - shared buffer hits: `361 -> 361`
- `ef_search=200`
  - execution time: `124.454ms -> 66.802ms`
  - shared buffer hits: `1273 -> 1273`

So the win is CPU-side, not I/O-side.

### Verified real-corpus `m=8` surface

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_noqjl_score.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=50.283ms p95=53.238ms  p99=55.862ms  mean=50.521ms min=46.541ms max=60.999ms  server_qps=19.79 wall=10.83s
m=8   ef_search=64   n=200   p50=54.331ms p95=91.486ms  p99=104.949ms mean=57.345ms min=48.861ms max=123.289ms server_qps=17.44 wall=13.77s
m=8   ef_search=100  n=200   p50=57.820ms p95=63.342ms  p99=70.617ms  mean=57.997ms min=51.008ms max=75.340ms  server_qps=17.24 wall=12.35s
m=8   ef_search=128  n=200   p50=60.312ms p95=64.450ms  p99=66.955ms  mean=60.150ms min=54.430ms max=69.580ms  server_qps=16.63 wall=14.13s
m=8   ef_search=160  n=200   p50=63.722ms p95=68.823ms  p99=70.548ms  mean=63.575ms min=56.444ms max=75.351ms  server_qps=15.73 wall=13.46s
m=8   ef_search=200  n=200   p50=68.254ms p95=77.307ms  p99=83.182ms  mean=68.260ms min=58.277ms max=85.272ms  server_qps=14.65 wall=15.80s
```

Artifact path:

- `/tmp/nfr1_real_10k_m8_noqjl_score.summary`

## Read

This slice is worth keeping.

Compared to the post-greedy canonical `m=8` surface from packet `256`, the
QJL-disabled 4-bit score fast path reduces mean latency by about `28% -> 45%`
across the measured `ef_search` range.

Against `NFR-001`, the lane is still not near closeout:

- requirement baseline: `p50 < 5ms` at `m=8, ef_search=40`
- current measured baseline: `p50 = 50.283ms`, `p99 = 55.862ms`

But this is another real C1 step-change, and it leaves the remaining work even
more clearly concentrated in the traversal layer rather than the per-candidate
score function.

## Exit criteria

- the active packet records the exact score-path change
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- the packet captures both microbench or hot-path evidence and the scan-level
  effect on C1
- if the fast path does not buy real scan latency, the packet says so plainly
