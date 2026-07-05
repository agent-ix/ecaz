# Task 28 IVF Quantizer Head-to-Head Smoke

## Scope

This packet records a first A10-oriented smoke comparison at head
`1a05603b2425fb74cb06362479ed71f2ce56ea46`.

This is not the full A10 gate. It covers one 10k fixture and two `nprobe`
points to expose current variant behavior after TurboQuant, PQ-FastScan, and
RaBitQ are all selectable in `ec_ivf`.

## Fixture

Local PG18 database `postgres`.

Base table and query source:

- `task28_ivf_postopt10k_n64w25_corpus`
- `task28_ivf_postopt10k_n64w25_queries`

The packet clones those rows into three isolated one-index surfaces:

- `task28_ivf_qcmp10k_turboquant`
- `task28_ivf_qcmp10k_pqfastscan`
- `task28_ivf_qcmp10k_rabitq`

All three indexes use:

- `nlists = 64`
- `nprobe = 64` index reloption, with runtime sweep `nprobe in {32,48}`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `rerank_width = 25`

Only `storage_format` differs.

## Build and Size

| variant | build time | index size |
| --- | ---: | ---: |
| TurboQuant | 21.722 s | 9416 kB |
| PQ-FastScan | 24.761 s | 1968 kB |
| RaBitQ | 22.143 s | 9416 kB |

PQ-FastScan is about 21% of the TurboQuant/RaBitQ index size on this fixture.

## Recall

100-query recall, `k=10`, force-index:

| variant | nprobe | recall@10 | ndcg@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: |
| TurboQuant | 32 | 0.9800 | 0.9981 | 61.99 ms |
| TurboQuant | 48 | 1.0000 | 1.0000 | 83.49 ms |
| PQ-FastScan | 32 | 0.3880 | 0.9079 | 32.94 ms |
| PQ-FastScan | 48 | 0.3890 | 0.9081 | 39.48 ms |
| RaBitQ | 32 | 0.9800 | 0.9981 | 1219.21 ms |
| RaBitQ | 48 | 1.0000 | 1.0000 | 1846.27 ms |

Interpretation:

- TurboQuant remains the only competitive high-recall choice in this smoke.
- PQ-FastScan is much smaller and faster, but current recall is not acceptable
  at this shape. The next PQ slice needs to investigate codebook training,
  rerank-width interaction, and whether the IVF scan should rerank a much wider
  prefilter for PQ-FastScan.
- RaBitQ matches TurboQuant recall here but is not scan-competitive in the
  current IVF integration. The likely next target is avoiding per-posting
  estimator overhead and checking whether this should use a different payload
  layout/profile before broader measurement.

## Latency

100-query latency, `k=10`, force-index:

| variant | nprobe | count | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| TurboQuant | 32 | 100 | 63.1 ms | 69.8 ms | 76.2 ms |
| TurboQuant | 48 | 100 | 82.6 ms | 89.8 ms | 94.3 ms |
| PQ-FastScan | 32 | 100 | 32.7 ms | 34.5 ms | 36.8 ms |
| PQ-FastScan | 48 | 100 | 40.2 ms | 46.3 ms | 50.7 ms |

RaBitQ latency was narrowed to 10 iterations at `nprobe=32` because the full
100-query/two-point run was already above a practical smoke window:

| variant | nprobe | count | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| RaBitQ | 32 | 10 | 1276.7 ms | 1407.3 ms | 1428.2 ms |

## Recommendation

Do not change `storage_format = 'auto'` from TurboQuant based on this smoke.
TurboQuant wins the only high-recall/latency tradeoff that is usable here.

The next useful work is not a default change. It is variant-specific:

- PQ-FastScan: recover recall while preserving the strong size/speed signal.
- RaBitQ: make scan scoring cheaper before repeating broad sweeps.
- A10 full gate: rerun on 10k and 25k with recall@100/NDCG@10/memory and
  matched-recall points after the variant-specific fixes.

## Artifacts

- `artifacts/build_quantizer_surfaces.sql`
- `artifacts/build_quantizer_surfaces.log`
- `artifacts/recall_turboquant.log`
- `artifacts/recall_pqfastscan.log`
- `artifacts/recall_rabitq.log`
- `artifacts/recall_rabitq_narrow.log`
- `artifacts/latency_turboquant.log`
- `artifacts/latency_pqfastscan.log`
- `artifacts/latency_rabitq_narrow.log`
- `artifacts/manifest.md`
