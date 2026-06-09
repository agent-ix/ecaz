# Task 94 Packet 025: Local PqFastScan Bench Matrix

This packet expands the local Task 94 evidence beyond the packet 024 smoke. It runs a suite-driven local matrix for IVF PqFastScan rerank-off at 10k/25k/100k and forced grouped-PQ DiskANN latency at 50k/100k.

Code under measurement is still checkpoint `187be1af1` (`Batch IVF PqFastScan scratch scoring`). This packet is measurement-only and does not add source code.

## Scope

- Local PG18 only: database `postgres`, socket `/home/peter/.pgrx`, port `28818`.
- Local Intel AVX2 only.
- No AWS and no GitHub CI were run.
- Suite config: `artifacts/task94-local-pqfastscan-matrix-suite.json`.
- Suite report: `artifacts/suite-report-cli.log`.
- Structured results: `artifacts/results.jsonl`.

## Validation

Suite audit passed:

```text
[suite:task94-local-pqfastscan-matrix] audit passed: 14 steps
```

Suite report result:

```text
steps: completed 14, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
```

## IVF Recall Parity

IVF PqFastScan batch-on recall exactly matches batch-off recall across all local fixture sizes:

| Fixture | nprobe | batch off recall / ndcg | batch on recall / ndcg |
| --- | ---: | --- | --- |
| 10k | 32 | `0.4620` / `0.9036` | `0.4620` / `0.9036` |
| 10k | 64 | `0.4660` / `0.9051` | `0.4660` / `0.9051` |
| 25k | 32 | `0.4870` / `0.9276` | `0.4870` / `0.9276` |
| 25k | 64 | `0.4900` / `0.9283` | `0.4900` / `0.9283` |
| 100k | 32 | `0.6350` / `0.9679` | `0.6350` / `0.9679` |
| 100k | 64 | `0.6360` / `0.9679` | `0.6360` / `0.9679` |

## IVF Latency

Local end-to-end latency is mixed rather than a clean win at every cell:

| Fixture | nprobe | batch off p50 / p95 / p99 | batch on p50 / p95 / p99 |
| --- | ---: | --- | --- |
| 10k | 32 | `2.87 ms` / `3.14 ms` / `3.25 ms` | `2.90 ms` / `3.21 ms` / `3.55 ms` |
| 10k | 64 | `4.58 ms` / `4.95 ms` / `5.35 ms` | `4.63 ms` / `4.96 ms` / `5.42 ms` |
| 25k | 32 | `5.54 ms` / `6.06 ms` / `6.50 ms` | `5.61 ms` / `6.18 ms` / `6.63 ms` |
| 25k | 64 | `10.0 ms` / `11.0 ms` / `15.4 ms` | `9.92 ms` / `10.8 ms` / `13.6 ms` |
| 100k | 32 | `18.0 ms` / `21.4 ms` / `27.5 ms` | `17.9 ms` / `22.9 ms` / `25.8 ms` |
| 100k | 64 | `34.8 ms` / `42.8 ms` / `49.6 ms` | `34.6 ms` / `41.7 ms` / `48.1 ms` |

Interpretation: the local batch path is correct and active. End-to-end p50 wins show up at larger/higher-nprobe cells, while smaller cells are within a small local overhead band. Graviton 4 SVE2 is still the important external closeout target.

## Direct Counter Rows

The suite result parser now carries direct `block_kernel_counters` rows, so Task 99-style evidence is present in `results.jsonl`.

| Surface / fixture | Label | ISA | kernel_candidates | scalar_candidates |
| --- | --- | --- | ---: | ---: |
| IVF 10k | `nprobe=32` | `avx2` | 2401600 | 0 |
| IVF 10k | `nprobe=32` | `scalar` | 0 | 7455 |
| IVF 10k | `nprobe=64` | `avx2` | 4992000 | 0 |
| IVF 10k | `nprobe=64` | `scalar` | 0 | 8000 |
| IVF 25k | `nprobe=32` | `avx2` | 6667840 | 0 |
| IVF 25k | `nprobe=32` | `scalar` | 0 | 7330 |
| IVF 25k | `nprobe=64` | `avx2` | 12496000 | 0 |
| IVF 25k | `nprobe=64` | `scalar` | 0 | 4000 |
| IVF 100k | `nprobe=32` | `avx2` | 24142560 | 0 |
| IVF 100k | `nprobe=32` | `scalar` | 0 | 7530 |
| IVF 100k | `nprobe=64` | `avx2` | 50000000 | 0 |
| DiskANN 50k | `list_size=64` | `avx2` | 6432 | 0 |
| DiskANN 50k | `list_size=64` | `scalar` | 0 | 145026 |
| DiskANN 50k | `list_size=128` | `avx2` | 6592 | 0 |
| DiskANN 50k | `list_size=128` | `scalar` | 0 | 240260 |
| DiskANN 100k | `list_size=64` | `avx2` | 6464 | 0 |
| DiskANN 100k | `list_size=64` | `scalar` | 0 | 161329 |
| DiskANN 100k | `list_size=128` | `avx2` | 6688 | 0 |
| DiskANN 100k | `list_size=128` | `scalar` | 0 | 273395 |

DiskANN evidence uses `ec_diskann.prefilter_kind=grouped_pq` to force the grouped-PQ surface on the existing local Task 67 DiskANN fixtures. That proves direct counter attribution for DiskANN grouped-PQ locally; it is not presented as a DiskANN speedup claim because most local traversal candidates still land in scalar tails.

## Review Ask

Please review packet 025 as the broader local benchmark evidence for Task 94 after the packet 024 production IVF PqFastScan call-site fix.

This packet does not ask for Task 94 completion. Remaining external gates are Graviton 4 SVE2 runtime dispatch/vector-length evidence and final full closeout benches when AWS testing is approved.
