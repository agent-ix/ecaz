# Review Request: Task 28 IVF Nlists Routing Grid

## Summary

This packet records a local `nlists x nprobe` routing grid after commit
`cc80443`. It keeps the DBPedia-derived 10k x 1536 anchor, `rerank =
'heap_f32'`, and small rerank widths, then varies centroid count and probe
fraction.

No DiskANN implementation or measurement is included.

## Fixture

- table: `task28_ivf_anchor10k1536_heap_corpus`
- exact top-10 source: `task28_ivf_anchor10k1536_heap_exact_top10` from packet 30038
- rows: 10,000
- dimensions: 1536
- queries: first 20 ordered anchor queries
- index: `ec_ivf`, rebuilt one grid point at a time
- `nlists`: 32, 64, 128
- probe fractions: 25%, 50%, 100%
- `rerank_width`: 25, 50
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Results

Recall/build summary:

| nlists | nprobe | width | probe fraction | build | recall@10 |
|---:|---:|---:|---:|---:|---:|
| 32 | 8 | 25 | 0.25 | `24.766 s` | `0.8350` |
| 32 | 8 | 50 | 0.25 | `24.697 s` | `0.8350` |
| 32 | 16 | 25 | 0.50 | `24.784 s` | `0.9400` |
| 32 | 16 | 50 | 0.50 | `24.975 s` | `0.9400` |
| 32 | 32 | 25 | 1.00 | `24.519 s` | `1.0000` |
| 32 | 32 | 50 | 1.00 | `24.861 s` | `1.0000` |
| 64 | 16 | 25 | 0.25 | `42.157 s` | `0.9400` |
| 64 | 16 | 50 | 0.25 | `42.383 s` | `0.9400` |
| 64 | 32 | 25 | 0.50 | `42.222 s` | `1.0000` |
| 64 | 32 | 50 | 0.50 | `42.282 s` | `1.0000` |
| 64 | 64 | 25 | 1.00 | `42.530 s` | `1.0000` |
| 64 | 64 | 50 | 1.00 | `42.380 s` | `1.0000` |
| 128 | 32 | 25 | 0.25 | `77.218 s` | `1.0000` |
| 128 | 32 | 50 | 0.25 | `77.306 s` | `1.0000` |
| 128 | 64 | 25 | 0.50 | `77.229 s` | `1.0000` |
| 128 | 64 | 50 | 0.50 | `77.924 s` | `1.0000` |
| 128 | 128 | 25 | 1.00 | `77.333 s` | `1.0000` |
| 128 | 128 | 50 | 1.00 | `76.327 s` | `1.0000` |

Latency loop:

| nlists | nprobe | width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 8 | 25 | `51.421 ms` | `67.886 ms` | `78.268 ms` | `54.348 ms` |
| 32 | 8 | 50 | `53.031 ms` | `68.036 ms` | `74.429 ms` | `55.234 ms` |
| 32 | 16 | 25 | `101.078 ms` | `117.615 ms` | `120.367 ms` | `102.101 ms` |
| 32 | 16 | 50 | `99.576 ms` | `112.549 ms` | `114.396 ms` | `100.319 ms` |
| 32 | 32 | 25 | `174.180 ms` | `179.704 ms` | `182.104 ms` | `175.199 ms` |
| 32 | 32 | 50 | `174.812 ms` | `184.001 ms` | `185.836 ms` | `176.272 ms` |
| 64 | 16 | 25 | `107.256 ms` | `117.004 ms` | `123.139 ms` | `108.135 ms` |
| 64 | 16 | 50 | `105.043 ms` | `118.549 ms` | `119.864 ms` | `97.228 ms` |
| 64 | 32 | 25 | `184.355 ms` | `205.062 ms` | `212.320 ms` | `188.215 ms` |
| 64 | 32 | 50 | `180.376 ms` | `192.687 ms` | `199.955 ms` | `182.853 ms` |
| 64 | 64 | 25 | `181.225 ms` | `188.242 ms` | `192.334 ms` | `172.584 ms` |
| 64 | 64 | 50 | `181.049 ms` | `198.910 ms` | `210.749 ms` | `185.383 ms` |
| 128 | 32 | 25 | `181.354 ms` | `184.835 ms` | `188.849 ms` | `181.976 ms` |
| 128 | 32 | 50 | `183.356 ms` | `233.627 ms` | `238.768 ms` | `197.882 ms` |
| 128 | 64 | 25 | `178.811 ms` | `192.455 ms` | `207.704 ms` | `181.502 ms` |
| 128 | 64 | 50 | `177.378 ms` | `184.561 ms` | `188.650 ms` | `178.274 ms` |
| 128 | 128 | 25 | `181.814 ms` | `207.960 ms` | `216.214 ms` | `187.074 ms` |
| 128 | 128 | 50 | `186.104 ms` | `204.342 ms` | `221.596 ms` | `191.170 ms` |

## Interpretation

On this 20-query 10k anchor, `nlists=32,nprobe=32,width=25` remains the best
full-recall point by p95 and build time. Larger centroid counts recover full
recall with a smaller probe fraction, but not with better latency at this
scale:

| point | recall@10 | p50 | p95 | build |
|---|---:|---:|---:|---:|
| `32/32,width=25` | `1.0000` | `174.180 ms` | `179.704 ms` | `24.519 s` |
| `64/32,width=50` | `1.0000` | `180.376 ms` | `192.687 ms` | `42.282 s` |
| `128/32,width=25` | `1.0000` | `181.354 ms` | `184.835 ms` | `77.218 s` |

The 64-list quarter-probe point matches 32-list half-probe recall (`0.9400`),
but has similar latency and materially higher build cost.

## Next Slice Recommendation

Keep `nlists=32` as the local default for the current 10k anchor and expand the
query sample before changing corpus size:

- run 100 ordered anchor queries for `32/16,width=50`, `32/32,width=25`, and
  `64/16,width=25`
- keep exact top-10 materialized packet-local
- only then decide whether a 50k local slice is worth the build cost

## Artifacts

- `artifacts/pg18-ivf-anchor10k1536-nlists-routing-grid.sql`
- `artifacts/pg18-ivf-anchor10k1536-nlists-routing-grid.log`
- `artifacts/pg18-active-during-nlists-grid.log`
- `artifacts/manifest.md`

## Validation

Packet-only change.

- `git diff --check`
