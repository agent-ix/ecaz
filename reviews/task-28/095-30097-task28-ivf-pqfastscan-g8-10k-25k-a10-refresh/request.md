# Task 28 IVF PQ-FastScan g8 10k/25k A10 Refresh

Packet 30096 recommended re-running the smaller A10 comparison with the
improved PQ-FastScan g8 shape instead of the initial PQ-FastScan default. This
packet refreshes the 10k and 25k local DBPedia slices.

## Fixture

Existing isolated surfaces were reused:

- 10k TurboQuant: `task28_ivf_qcmp10k_turboquant`
- 10k PQ-FastScan g8: `task28_ivf_pqg10k_g8`
- 25k TurboQuant: `task28_ivf_postopt25k_n64w25`
- 25k PQ-FastScan g8: `task28_ivf_pqg25k_g8`

The PQ-FastScan indexes were temporarily set to `rerank_width=750` for the
refresh. TurboQuant was first measured at its current `rerank_width=25`, then
temporarily set to `rerank_width=750` for a matched-width comparison. All four
indexes were restored to `rerank_width=25` after the measurements.

## Current Width Comparison

This compares the current tuned operating surfaces: TurboQuant width25 versus
PQ-FastScan g8 width750.

| corpus | profile | nprobe | recall@10 | recall@100 | mean q-time @10 |
|---|---|---:|---:|---:|---:|
| 10k | TurboQuant w25 | 48 | 1.0000 | 0.2500 | 85.70 ms |
| 10k | PQ-FastScan g8 w750 | 48 | 0.9910 | 0.9360 | 86.80 ms |
| 25k | TurboQuant w25 | 48 | 0.9990 | 0.2500 | 204.09 ms |
| 25k | PQ-FastScan g8 w750 | 48 | 0.9940 | 0.9256 | 152.36 ms |

The TurboQuant recall@100 value is capped by width25; it should not be read as
a quality failure. It means the current TurboQuant operating profile cannot
answer a top-100 recall question without widening rerank.

## Matched Width750 Comparison

This compares both profiles at `rerank_width=750`.

| corpus | profile | nprobe | recall@10 | recall@100 | mean q-time @10 | mean q-time @100 |
|---|---|---:|---:|---:|---:|---:|
| 10k | TurboQuant w750 | 48 | 1.0000 | 0.9966 | 120.89 ms | 164.02 ms |
| 10k | PQ-FastScan g8 w750 | 48 | 0.9910 | 0.9360 | 86.80 ms | 124.21 ms |
| 25k | TurboQuant w750 | 48 | 0.9990 | 0.9929 | 237.92 ms | 270.48 ms |
| 25k | PQ-FastScan g8 w750 | 48 | 0.9940 | 0.9256 | 152.36 ms | 187.78 ms |

Matched-width latency at `k=10`, `nprobe=48`:

| corpus | profile | p50 | p95 | p99 |
|---|---|---:|---:|---:|
| 10k | TurboQuant w750 | 118.8 ms | 147.2 ms | 160.8 ms |
| 10k | PQ-FastScan g8 w750 | 85.4 ms | 104.4 ms | 117.0 ms |
| 25k | TurboQuant w750 | 231.5 ms | 271.3 ms | 284.6 ms |
| 25k | PQ-FastScan g8 w750 | 145.7 ms | 171.9 ms | 194.1 ms |

## Interpretation

PQ-FastScan g8 is clearly faster and smaller at matched width750:

- 10k p48 p50: `85.4 ms` versus TurboQuant `118.8 ms`.
- 25k p48 p50: `145.7 ms` versus TurboQuant `231.5 ms`.
- 10k index size: PQ g8 `2448 kB` versus TurboQuant `9416 kB`.
- 25k index size: PQ g8 `5176 kB` versus TurboQuant `22 MB`.

TurboQuant still wins recall on 10k/25k, especially recall@100:

- 10k p48 recall@100: TurboQuant `0.9966` versus PQ g8 `0.9360`.
- 25k p48 recall@100: TurboQuant `0.9929` versus PQ g8 `0.9256`.

That means the 100k result from packet 30096 should not be generalized as a
global default decision for all corpus sizes. PQ-FastScan g8 is the leading
100k local lane, but TurboQuant remains the higher-recall smaller-corpus
profile at matched rerank width.

## Recommendation

Keep the current conservative default posture:

- Recommend explicit `storage_format='pq_fastscan', pq_group_size=8` for the
  100k high-dimensional local lane where packet 30096 shows it wins on speed
  and size at comparable recall.
- Do not change `storage_format='auto'` globally from this branch yet.
- If `auto` changes later, make it dimension/corpus-size aware or gate it on a
  broader A10 matrix; a global switch would regress the 10k/25k recall@100
  story.

The next useful PQ slice is recall recovery at smaller corpora: test whether
`nlists=128` or a wider PQ rerank frontier closes the 10k/25k recall@100 gap
without giving up the latency advantage.

## Artifacts

See `artifacts/manifest.md`.
