# Task 111e Packet 006 Artifact Manifest

- head SHA: `dda25be9068fe563c67885b03a6e0d15506bdd12`
- task bucket: `reviews/task-111e/006-final-gate`
- purpose: final candidate-frontier, heap-f32 coarse-rerank, EXPLAIN, storage, and promotion-gate evidence
- generated: `2026-06-18T18:05:20Z`
- lane / fixture: local PG18, database `task111e_coarse_rerank`, socket `/home/peter/.pgrx`, port `28818`
- profile / dimensions: `ec_ivf`, 1536 dimensions, `k=10`, 100 queries, seed 42
- surfaces: isolated one-index-per-corpus-table prefixes in one benchmark database
- storage format under gate: `storage_format=coarse_rerank`, `coarse_format=rabitq`, `coarse_bits=1`, `rerank=heap_f32`, `rerank_placement=table`, `rerank_format=f32`, `rerank_width=100`
- candidate-frontier comparison surface: RaBitQ-1 dense page-local postings with `rerank=off`

## Commands

```text
cargo build --release -p ecaz-cli --bin ecaz

target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-111e/006-final-gate/artifacts/install-ecaz-pg18-release.log

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-audit.log bench suite audit --config reviews/task-111e/006-final-gate/artifacts/task111e-final-gate-suite.json --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-run.log bench suite run --config reviews/task-111e/006-final-gate/artifacts/task111e-final-gate-suite.json --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results.jsonl

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-audit-r2.log bench suite audit --config reviews/task-111e/006-final-gate/artifacts/task111e-final-gate-suite.json --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-run-explain-r2.log bench suite run --config reviews/task-111e/006-final-gate/artifacts/task111e-final-gate-suite.json --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818 --only-tag explain --manifest-output reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-explain-r2.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results-explain-r2.jsonl

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-run-frontier-r2.log bench suite run --config reviews/task-111e/006-final-gate/artifacts/task111e-final-gate-suite.json --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818 --only-tag frontier --manifest-output reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-frontier-r2.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results-frontier-r2.jsonl

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-status-main.log bench suite status --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest.json

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-report-main.log bench suite report --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results-main-report.jsonl

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-status-explain-r2.log bench suite status --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-explain-r2.json

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-report-explain-r2.log bench suite report --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-explain-r2.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results-explain-report-r2.jsonl

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-status-frontier-r2.log bench suite status --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-frontier-r2.json

target/release/ecaz --log-file reviews/task-111e/006-final-gate/artifacts/suite-report-frontier-r2.log bench suite report --manifest reviews/task-111e/006-final-gate/artifacts/suite/suite-manifest-frontier-r2.json --results-output reviews/task-111e/006-final-gate/artifacts/suite/results-frontier-report-r2.jsonl
```

## Run Notes

The first full suite completed load, storage, recall, and latency, then failed
at the first EXPLAIN step because the generated SQL referred to
`task111e_006_50k_coarse_rerank_idx` instead of the actual
`task111e_006_50k_coarse_rerank_coarse_rerank_idx`. The suite config was fixed
with explicit EXPLAIN index names, audited again, and rerun with `--only-tag
explain` plus `--only-tag frontier`.

Because the fixed config hash no longer matched the first run manifest, the
packet uses the first manifest for the successful load/storage/recall/latency
rows and the r2 manifests for corrected EXPLAIN and frontier rows.

## Artifacts

| Artifact | Result |
| --- | --- |
| `task111e-final-gate-suite.json` | Suite config for 50k/100k coarse-rerank, EXPLAIN, and frontier sweeps. |
| `install-ecaz-pg18-release.log` | PG18 install of release build succeeded. |
| `suite-audit.log` | Initial audit passed: 30 steps. |
| `suite-run.log` | Initial run: 14 succeeded before first EXPLAIN failed due index-name config. |
| `suite-status-main.log` | Status for initial run: 14 completed, 1 failed, 15 stale after config fix. |
| `suite-report-main.log` | Parsed load, storage, recall, and latency rows from initial run. |
| `suite-audit-r2.log` | Re-audit after explicit EXPLAIN index names passed: 30 steps. |
| `suite-run-explain-r2.log` | Corrected EXPLAIN-only run completed. |
| `suite-status-explain-r2.log` | EXPLAIN status: 4 completed, 0 failed. |
| `suite-report-explain-r2.log` | Parsed EXPLAIN planner rows. |
| `suite-run-frontier-r2.log` | Frontier-only run completed. Load steps were selected by tag and skipped/reused existing fixtures. |
| `suite-status-frontier-r2.log` | Frontier status: 14 completed, 0 failed. |
| `suite-report-frontier-r2.log` | Parsed 50k/100k frontier rows. |
| `suite/results-main-report.jsonl` | Structured parsed load/storage/recall/latency rows. |
| `suite/results-explain-r2.jsonl` | Structured EXPLAIN run results. |
| `suite/results-explain-report-r2.jsonl` | Structured parsed EXPLAIN planner rows. |
| `suite/results-frontier-r2.jsonl` | Structured frontier run results. |
| `suite/results-frontier-report-r2.jsonl` | Structured parsed frontier rows. |
| `suite/*.log` | Packet-local per-step raw logs for load, storage, recall, latency, EXPLAIN, and frontier steps. |

## Storage And Build

| Prefix | Rows | Build index | Total load | EC IVF index size | EC IVF bytes/row |
| --- | ---: | ---: | ---: | ---: | ---: |
| `task111e_006_50k_coarse_rerank` | 50,000 | 3.61 s | 44.85 s | 11.6 MiB | 243.3 B |
| `task111e_006_100k_coarse_rerank` | 100,000 | 7.07 s | 93.89 s | 22.5 MiB | 235.8 B |

The table totals include raw f32 source vectors and are therefore not index
storage measurements: 806.5 MiB at 50k and 1.6 GiB at 100k.

## Heap-F32 Coarse-Rerank Recall And Latency

| Corpus | Rerank width | nprobe | Recall@10 | NDCG@10 | Recall mean q-time | Latency p50 | Latency p95 | Latency p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 50 | 32 | 0.9940 | 0.9997 | 7.06 ms | 6.60 ms | 7.67 ms | 8.52 ms |
| 50k | 50 | 64 | 0.9980 | 1.0000 | 10.53 ms | 10.1 ms | 11.0 ms | 15.3 ms |
| 50k | 100 | 32 | 0.9960 | 0.9997 | 8.53 ms | 7.91 ms | 8.86 ms | 11.0 ms |
| 50k | 100 | 64 | 1.0000 | 1.0000 | 12.49 ms | 11.6 ms | 12.1 ms | 15.0 ms |
| 100k | 50 | 32 | 0.9710 | 0.9969 | 11.05 ms | 11.1 ms | 12.4 ms | 13.9 ms |
| 100k | 50 | 64 | 0.9980 | 1.0000 | 17.70 ms | 17.2 ms | 18.2 ms | 23.0 ms |
| 100k | 100 | 32 | 0.9730 | 0.9969 | 12.63 ms | 12.3 ms | 15.2 ms | 17.6 ms |
| 100k | 100 | 64 | 1.0000 | 1.0000 | 19.12 ms | 19.9 ms | 24.6 ms | 26.5 ms |

## Candidate Frontier

Rows use the RaBitQ-1 dense page-local candidate frontier with `rerank=off`,
then score `f32` table-side sidecar vectors in free-read mode.

| Corpus | candidate_k | nprobe | Recall@10 | Total-bound p50 | Bytes touched p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 50k | 25 | 32 | 0.9750 | 6.826 ms | 150.00 KiB |
| 50k | 50 | 32 | 0.9940 | 8.572 ms | 300.00 KiB |
| 50k | 100 | 32 | 0.9960 | 10.597 ms | 600.00 KiB |
| 50k | 1000 | 32 | 0.9960 | 52.153 ms | 5.86 MiB |
| 50k | 50 | 64 | 0.9980 | 14.673 ms | 300.00 KiB |
| 50k | 100 | 64 | 1.0000 | 16.428 ms | 600.00 KiB |
| 100k | 25 | 32 | 0.9530 | 13.808 ms | 150.00 KiB |
| 100k | 50 | 32 | 0.9710 | 14.498 ms | 300.00 KiB |
| 100k | 100 | 32 | 0.9730 | 17.040 ms | 600.00 KiB |
| 100k | 1000 | 32 | 0.9730 | 60.550 ms | 5.86 MiB |
| 100k | 50 | 64 | 0.9980 | 25.117 ms | 300.00 KiB |
| 100k | 100 | 64 | 1.0000 | 27.348 ms | 600.00 KiB |
| 100k | 1000 | 64 | 1.0000 | 69.431 ms | 5.86 MiB |

At 100k, widening from 100 to 1000 candidates does not improve nprobe32 recall
above 0.9730, while total-bound p50 rises from 17.040 ms to 60.550 ms.

## EXPLAIN Counters

All rows used `EXPLAIN (ANALYZE, VERBOSE, FORMAT JSON, ecaz)` at nprobe 32.

| Corpus | Rerank width | Posting pages | Dense postings visited | Candidates scored | Rerank rows | Heap blocks fetched | Approx scan | Exact rerank | Execution |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 50 | 677 | 23,904 | 285 | 50 | 36 | 4,592 us | 1,993 us | 8.318 ms |
| 50k | 100 | 677 | 23,904 | 535 | 100 | 54 | 4,756 us | 4,570 us | 11.099 ms |
| 100k | 50 | 1,189 | 42,171 | 322 | 50 | 33 | 7,452 us | 2,162 us | 11.255 ms |
| 100k | 100 | 1,189 | 42,171 | 538 | 100 | 65 | 7,361 us | 3,898 us | 13.028 ms |

The same logs also expose centroid scores, selected lists, candidates inserted,
candidates emitted, row/dense/columnar posting split, and heap TIDs scored.

## Interpretation

- The gated `coarse_rerank` heap-f32 path works end to end and is compact on
  index storage: 11.6 MiB at 50k and 22.5 MiB at 100k.
- 50k reaches a useful high-recall point at nprobe32/width50: 0.9940 recall at
  6.60 ms p50, with width100 gaining only 0.002 recall for 7.91 ms p50.
- 100k needs nprobe64 for high recall. nprobe32 stays at 0.9710 to 0.9730 even
  when the sidecar candidate budget grows to 1000, so the limiter is the
  coarse frontier/list coverage rather than rerank width.
- nprobe64/width50 reaches 0.9980 recall at 17.2 ms p50; nprobe64/width100
  reaches 1.0000 recall at 19.9 ms p50.
- Packet 005 carries the compact representation decision: table-side `f16`
  preserves `f32` recall/NDCG while halving sidecar bytes, and `rabitq8` is
  rejected for the immediate high-recall path because recall falls to 0.9460.
- Packet 005 also separates true index-side rerank placement as follow-up
  scope; this packet keeps table/heap-side placement.

## Recommendation

Iterate, do not promote `coarse_rerank` as a default yet.

The 50k slice is promising and the 100k nprobe64 slice demonstrates that the
heap-f32 rerank path can recover near-exact quality, but the 100k nprobe32
plateau shows that the current RaBitQ-1 coarse frontier loses candidates before
rerank. Next work should improve frontier coverage or cost, for example with
residual/RaBitQ-2 coarse scoring, adaptive nprobe, or better bounds, before
promotion.
