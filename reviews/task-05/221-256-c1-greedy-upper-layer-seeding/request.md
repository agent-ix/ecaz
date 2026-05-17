# Review Request: C1 Greedy Upper-Layer Seeding

## Context

Packet `255` showed that fast hash state is worth keeping but is not the next
big latency unlock. After the score-cache and fast-hash slices, the canonical
real `10k` `m=8` surface still sits around:

- `ef_search=40`: mean `88.360ms`
- `ef_search=200`: mean `174.147ms`

The post-fast-hash representative hot-path probe still shows large rescan
seeding cost:

- `ef_search=40`
  - upper-layer seed elapsed: `24.271ms`
  - layer-0 seed elapsed: `14.207ms`
- `ef_search=200`
  - upper-layer seed elapsed: `61.339ms`
  - layer-0 seed elapsed: `66.907ms`

That makes upper-layer seed search one of the remaining highest-signal C1
targets.

## Problem

The scan runtime currently uses a full result-window search across every upper
layer during `amrescan` seeding. That is more expensive than the classic HNSW
pattern, which greedily descends upper layers to a single best local optimum
and only opens the wider beam at layer 0.

The rest of the codebase already trusts greedy upper-layer descent for:

- insert search
- vacuum repair search

So the scan runtime is now the outlier.

## Planned work

1. Switch scan-time upper-layer seeding from per-layer result-window search to
   cached greedy descent.
2. Keep layer-0 search behavior unchanged.
3. Re-run the existing validation suite, including the recall gate already
   exercised by `cargo pgrx test pg17`.
4. Re-run the representative hot-path probe and canonical `m=8` verified
   surface to see whether the upper-layer bucket collapses materially without
   compromising behavior.

## Progress

The code checkpoint for the greedy shift is now in place and green under the
required validation gate:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The concrete code change is narrow:

- expose `graph::greedy_descend_with_successors` to the scan runtime
- replace scan-time upper-layer result-window seeding with greedy descent to a
  single candidate
- keep the layer-0 beam search unchanged, seeded from that single upper-layer
  result

Measurement is now complete.

## Results

### Representative hot-path probe

Compared to the post-fast-hash baseline from packet `255`:

- `ef_search=40`
  - upper-layer seed elapsed: `24.271ms -> 3.722ms`
  - layer-0 seed elapsed: `14.207ms -> 15.802ms`
  - candidate score calls: `470 -> 241`
  - candidate score elapsed: `31.530ms -> 16.217ms`
- `ef_search=200`
  - upper-layer seed elapsed: `61.339ms -> 3.742ms`
  - layer-0 seed elapsed: `66.907ms -> 87.147ms`
  - candidate score calls: `1574 -> 997`
  - candidate score elapsed: `105.936ms -> 76.667ms`

So the greedy shift did exactly what it was supposed to do: it nearly removed
upper-layer seeding from the hot path. Layer-0 search becomes more prominent,
but total seed+score work still drops materially.

### Representative SQL

On the shared canonical `tqhnsw_real_10k_m8_idx` probe for query `id=10000`:

- `ef_search=40`
  - execution time: `80.683ms -> 62.285ms`
  - shared buffer hits: `668 -> 361`
- `ef_search=200`
  - execution time: `172.001ms -> 124.454ms`
  - shared buffer hits: `2141 -> 1273`

### Verified real-corpus `m=8` surface

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_greedy_upper.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=69.562ms  p95=79.380ms  p99=83.530ms  mean=69.855ms  min=55.491ms  max=84.006ms  server_qps=14.32 wall=14.71s
m=8   ef_search=64   n=200   p50=79.898ms  p95=90.619ms  p99=98.609ms  mean=79.753ms  min=62.327ms  max=104.349ms server_qps=12.54 wall=16.71s
m=8   ef_search=100  n=200   p50=92.714ms  p95=106.607ms p99=116.384ms mean=92.465ms  min=71.284ms  max=117.682ms server_qps=10.81 wall=20.62s
m=8   ef_search=128  n=200   p50=102.767ms p95=114.942ms p99=119.196ms mean=101.467ms min=79.701ms  max=134.937ms server_qps=9.86  wall=21.02s
m=8   ef_search=160  n=200   p50=113.216ms p95=129.379ms p99=141.886ms mean=112.132ms min=87.779ms  max=162.703ms server_qps=8.92  wall=23.18s
m=8   ef_search=200  n=200   p50=126.960ms p95=145.479ms p99=150.469ms mean=124.238ms min=93.581ms  max=156.663ms server_qps=8.05  wall=25.57s
```

Artifact path:

- `/tmp/nfr1_real_10k_m8_greedy_upper.summary`

## Read

This slice is worth keeping.

Compared to the post-fast-hash canonical `m=8` surface from packet `255`, the
greedy upper-layer shift reduces mean latency by about `21% -> 29%` across the
entire measured `ef_search` range.

Against `NFR-001`, the result is still not close to a closeout:

- requirement baseline: `p50 < 5ms` at `m=8, ef_search=40`
- current measured baseline: `p50 = 69.562ms`, `p99 = 83.530ms`

The next C1 target is no longer upper-layer routing. The remaining dominant
runtime buckets are layer-0 search work and the scoring it still drives.

## Exit criteria

- a pushed checkpoint materially reduces upper-layer seed time on the real C1
  path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records measured before/after evidence, including whether the
  greedy shift was worth keeping
