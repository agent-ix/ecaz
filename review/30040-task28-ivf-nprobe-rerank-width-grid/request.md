# Review Request: Task 28 IVF Nprobe x Rerank Width Grid

## Summary

This packet records a local `nprobe x rerank_width` grid after commit
`697f49e`. It keeps the DBPedia-derived 10k x 1536 anchor, `nlists = 32`,
`training_sample_rows = 2000`, and `rerank = 'heap_f32'`.

No DiskANN implementation or measurement is included.

## Fixture

- table: `task28_ivf_anchor10k1536_heap_corpus`
- exact top-10 source: `task28_ivf_anchor10k1536_heap_exact_top10` from packet 30038
- rows: 10,000
- dimensions: 1536
- queries: first 20 ordered anchor queries
- index: `ec_ivf`, rebuilt one grid point at a time
- `nlists = 32`
- `nprobe`: 4, 8, 16, 32
- `rerank_width`: 25, 50, 100, 200
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Recall

On this 20-query slice, recall is controlled by routing breadth (`nprobe`), not
by the tested rerank widths:

| nprobe | rerank_width | recall@10 | exact hits |
|---:|---:|---:|---:|
| 4 | 25 | `0.6850` | 137/200 |
| 4 | 50 | `0.6850` | 137/200 |
| 4 | 100 | `0.6850` | 137/200 |
| 4 | 200 | `0.6850` | 137/200 |
| 8 | 25 | `0.8350` | 167/200 |
| 8 | 50 | `0.8350` | 167/200 |
| 8 | 100 | `0.8350` | 167/200 |
| 8 | 200 | `0.8350` | 167/200 |
| 16 | 25 | `0.9400` | 188/200 |
| 16 | 50 | `0.9400` | 188/200 |
| 16 | 100 | `0.9400` | 188/200 |
| 16 | 200 | `0.9400` | 188/200 |
| 32 | 25 | `1.0000` | 200/200 |
| 32 | 50 | `1.0000` | 200/200 |
| 32 | 100 | `1.0000` | 200/200 |
| 32 | 200 | `1.0000` | 200/200 |

## Latency

20-query latency loop:

| nprobe | rerank_width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|
| 4 | 25 | `36.803 ms` | `58.175 ms` | `82.245 ms` | `39.481 ms` |
| 4 | 50 | `33.666 ms` | `47.603 ms` | `49.002 ms` | `34.708 ms` |
| 4 | 100 | `35.971 ms` | `50.253 ms` | `51.351 ms` | `37.140 ms` |
| 4 | 200 | `39.079 ms` | `55.422 ms` | `57.220 ms` | `41.425 ms` |
| 8 | 25 | `54.673 ms` | `70.433 ms` | `71.631 ms` | `55.810 ms` |
| 8 | 50 | `51.854 ms` | `69.077 ms` | `72.197 ms` | `54.223 ms` |
| 8 | 100 | `55.424 ms` | `69.791 ms` | `74.188 ms` | `56.940 ms` |
| 8 | 200 | `60.731 ms` | `85.156 ms` | `96.363 ms` | `65.403 ms` |
| 16 | 25 | `98.070 ms` | `113.008 ms` | `116.266 ms` | `98.551 ms` |
| 16 | 50 | `98.683 ms` | `132.867 ms` | `133.369 ms` | `104.814 ms` |
| 16 | 100 | `97.893 ms` | `108.554 ms` | `110.344 ms` | `97.971 ms` |
| 16 | 200 | `102.973 ms` | `118.426 ms` | `123.167 ms` | `105.237 ms` |
| 32 | 25 | `171.101 ms` | `184.073 ms` | `198.929 ms` | `173.657 ms` |
| 32 | 50 | `178.186 ms` | `228.625 ms` | `230.652 ms` | `189.286 ms` |
| 32 | 100 | `173.411 ms` | `178.811 ms` | `179.668 ms` | `174.285 ms` |
| 32 | 200 | `180.555 ms` | `183.631 ms` | `183.869 ms` | `180.591 ms` |

## Build Time

Build time was stable across grid points, roughly 24.4-25.2 seconds per index.
`rerank_width` and `nprobe` are scan-time controls for this implementation.

## Interpretation

For this local anchor, `rerank_width = 25` is already enough once the right
posting lists are probed. The current recall/latency frontier is:

| point | recall@10 | p50 | p95 |
|---|---:|---:|---:|
| `nprobe=8,width=50` | `0.8350` | `51.854 ms` | `69.077 ms` |
| `nprobe=16,width=100` | `0.9400` | `97.893 ms` | `108.554 ms` |
| `nprobe=32,width=100` | `1.0000` | `173.411 ms` | `178.811 ms` |

The next tuning work should focus on list routing and centroid count. Width can
start at 25 or 50 for this corpus unless a broader query sample shows boundary
misses.

## Next Slice Recommendation

Run `nlists x nprobe` with a fixed small rerank width:

- `nlists`: 32, 64, 128
- `nprobe`: approximately 25%, 50%, and 100% of lists
- `rerank_width`: 25 and 50

Keep the first pass on the 10k x 1536 anchor, then repeat the best few points on
a larger local slice if build time is still manageable.

## Artifacts

- `artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.sql`
- `artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid.log` (failed first run; alias shadowing, no claims)
- `artifacts/pg18-ivf-anchor10k1536-nprobe-rerank-width-grid-rerun.log`
- `artifacts/manifest.md`

## Validation

Packet-only change after the prior code checkpoint.

- `git diff --check`
