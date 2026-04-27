# Review Request: Task 28 IVF 100-Query Midprobe Check

## Summary

This packet checks middle-ground `nprobe = 24` points against the 100-query
exact truth table from packet 30042. The goal is to see whether a partial probe
can close the `0.9800` recall gap without paying the full `nprobe = 32` latency.

No DiskANN implementation or measurement is included.

## Fixture

- corpus table: `task28_ivf_anchor10k1536_heap_corpus`
- query table: `task28_ivf_anchor10k1536_queries100`
- exact truth: `task28_ivf_anchor10k1536_exact100_top10`
- rows: 10,000
- dimensions: 1536
- queries: 100
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Results

Candidate points:

| nlists | nprobe | width | build | materialize | recall@10 | hits |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 24 | 25 | `24.340 s` | `14.616 s` | `0.9980` | 998/1000 |
| 32 | 24 | 50 | `24.467 s` | `14.830 s` | `0.9980` | 998/1000 |
| 64 | 24 | 25 | `42.328 s` | `14.673 s` | `0.9980` | 998/1000 |
| 64 | 24 | 50 | `41.558 s` | `15.704 s` | `0.9980` | 998/1000 |

Latency loop:

| nlists | nprobe | width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 24 | 25 | `135.073 ms` | `146.331 ms` | `154.755 ms` | `127.040 ms` |
| 32 | 24 | 50 | `137.423 ms` | `149.456 ms` | `151.791 ms` | `128.767 ms` |
| 64 | 24 | 25 | `143.789 ms` | `153.870 ms` | `160.552 ms` | `144.614 ms` |
| 64 | 24 | 50 | `146.575 ms` | `169.468 ms` | `184.034 ms` | `139.907 ms` |

## Interpretation

`nprobe=24` is a strong middle point but not exact-oracle quality on the
100-query sample. It recovers 998/1000 exact top-10 hits, with p50/p95 around
`135/146 ms` for `nlists=32,width=25`.

Compared with packet 30042:

| point | recall@10 | p50 | p95 |
|---|---:|---:|---:|
| `32/16,width=50` | `0.9800` | `100.013 ms` | `120.946 ms` |
| `32/24,width=25` | `0.9980` | `135.073 ms` | `146.331 ms` |
| `32/32,width=25` | `1.0000` | `177.806 ms` | `202.947 ms` |

For initial local tuning, this gives a clear latency/recall ladder. The
reasonable next local default candidates are:

- faster/high recall: `nlists=32,nprobe=24,rerank_width=25`
- exact-oracle on this sample: `nlists=32,nprobe=32,rerank_width=25`

## Next Slice Recommendation

Stop broadening knobs on the 10k anchor for now. The next useful slice is a
larger local corpus check for just the two candidates above, reusing the
100-query query set if practical. If a 50k build is too slow locally, try 25k
first and record build time explicitly.

## Artifacts

- `artifacts/pg18-ivf-anchor10k1536-anchor100-midprobe.sql`
- `artifacts/pg18-ivf-anchor10k1536-anchor100-midprobe.log`
- `artifacts/manifest.md`

## Validation

Packet-only change.

- `git diff --check`
