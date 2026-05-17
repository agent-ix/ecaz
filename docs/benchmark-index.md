# Benchmark Packet Index

This page is a directory of packet-backed benchmark surfaces.

Use [benchmarks.md](benchmarks.md) for the current benchmark tables and selected
results. Use [Benchmark Reporting Standard](benchmark-reporting-standard.md) for
the required fields in new AM, quantizer, storage-format, and option-set
comparisons. Use this page when you need to find the underlying packet quickly.

Only link measured or intentionally scaffolded benchmark packets here. If a
benchmark lane does not yet have a packet, leave it out of this index rather
than inventing an empty result.

## DiskANN

| Lane | Corpus / fixture | Packet |
| --- | --- | --- |
| Initial local readiness cross-engine sweep | real10K, isolated PG18 sweep | `review/11109-task29d-final-readiness/` |
| NEON exact rerank kernel A/B | synth10K, real10K, real10K_w800 | `review/30204-task29-diskann-m5-neon-rerank/` |
| Heap-TID rerank fetch A/B | real10K_w800 | `review/30205-task29-diskann-m5-rerank-heap-order/` |
| Heap-block prefetch warm-cache A/B | real10K_w800 | `review/30206-task29-diskann-m5-rerank-prefetch/` |
| M5 decision summary | packet rollup only | `review/30207-task29-diskann-m5-decision/` |
| Build-time scalar vs NEON A/B | real10K | `review/30208-task29-diskann-m5-build-neon-followup/` |
| Cold-cache prefetch 100K A/B | real100K | `review/30209-task29-diskann-m5-cold-cache-100k/` |
| Final post-M5 cross-engine refresh | real10K, isolated PG18 sweep | `review/30210-task32-m5-diskann-final-cross-engine-refresh/` |

## IVF

| Lane | Corpus / fixture | Packet |
| --- | --- | --- |
| Local landing status | 10K / 25K / 100K / 990K directional summary | `review/30151-task28-ivf-local-landing-status/` |
| 100K current build | 100K selected point | `review/30119-task28-ivf-a9-100k-current-build/` |
| M5 current candidate decision | 100K balanced + quality points | `review/30203-task31-current-m5-candidate-decision/` |

## Quantizer And Storage Candidates

| Lane | Corpus / fixture | Packet |
| --- | --- | --- |
| IVF TurboQuant / PQ-FastScan / RaBitQ head-to-head | 10K smoke | `review/30084-task28-ivf-quantizer-headtohead-smoke/` |
| IVF current auto recommendation | packet synthesis over 10K and 100K evidence | `review/30096-task28-ivf-current-auto-recommendation/` |
| IVF RaBitQ A10 fill | 10K / 25K local RaBitQ rows | `review/30144-task28-ivf-a10-rabitq-fill/` |
| SPIRE RaBitQ remote endpoint gate | PG18 loopback remote-serving readiness | `review/30709-spire-remote-heap-endpoint-gate/` |
| SPIRE tuple transport measurement | PG18 loopback, `storage_format=rabitq` | `review/30975-spire-tuple-transport-measurement/` |

## HNSW

| Lane | Corpus / fixture | Packet |
| --- | --- | --- |
| Local reference row used in DiskANN comparison | real10K, isolated PG18 sweep | `review/11109-task29d-final-readiness/` |

## Notes

- Packet-local `artifacts/manifest.md` files are the source of truth for
  commands, SHAs, hardware, cache state, and cited result lines.
- When adding a new benchmark packet, update this index and
  [benchmarks.md](benchmarks.md) together.
- Candidate comparison packets should follow
  [Benchmark Reporting Standard](benchmark-reporting-standard.md).
