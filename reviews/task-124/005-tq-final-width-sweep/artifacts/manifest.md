# Task 124 Packet 005 Artifact Manifest

- head SHA: `e5c1bf254360f45ec088bea82550f9a58238a901`
- task bucket: `reviews/task-124/005-tq-final-width-sweep`
- timestamp: `2026-06-29T02:06:00Z` through local continuation
- lane: local PG18 release build on `/Users/peter/.pgrx`, database `tqvector_bench`
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_{corpus,queries}.tsv`
- runner: `ecaz bench suite`
- isolation: one index per table/prefix

## Artifact Inventory

Decision sweep:

- config: `artifacts/task124-tq-final-width-100k-suite.json`
- suite manifest: `artifacts/final-width-100k-manifest.json`
- suite results: `artifacts/final-width-100k-results.jsonl`
- run log: `artifacts/final-width-100k-run.log`
- per-step load/recall/latency/storage/explain logs under `artifacts/`

10k / 50k / 100k A/B:

- config: `artifacts/task124-tq-final15-ab-10-50-100-suite.json`
- suite manifest: `artifacts/final15-ab-manifest.json`
- suite results: `artifacts/final15-ab-results.jsonl`
- run log: `artifacts/final15-ab-run.log`
- per-step load/recall/latency/storage logs under `artifacts/final15-ab/`

Generated truth caches are intentionally untracked and should not be committed.

## Provenance Note

The benchmark binary was the release `ecaz` already built for Task 124 packet 004. It includes the TQ scan/build behavior under review; packet 004 later added a CLI profile-only known-reloption cleanup. Some packet 005 load logs therefore still warn that `stage2_final_rerank_width` is not listed as a known CLI reloption. The loader passed the reloption through verbatim and the scans record the requested `ec_ivf.stage2_final_rerank_width` session GUC.

## Commands

100k decision sweep:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_bench bench suite run --config reviews/task-124/005-tq-final-width-sweep/artifacts/task124-tq-final-width-100k-suite.json --artifact-dir reviews/task-124/005-tq-final-width-sweep/artifacts --manifest-output reviews/task-124/005-tq-final-width-sweep/artifacts/final-width-100k-manifest.json --results-output reviews/task-124/005-tq-final-width-sweep/artifacts/final-width-100k-results.jsonl --log-file reviews/task-124/005-tq-final-width-sweep/artifacts/final-width-100k-run.log
```

10k / 50k / 100k A/B:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_bench bench suite run --config reviews/task-124/005-tq-final-width-sweep/artifacts/task124-tq-final15-ab-10-50-100-suite.json --artifact-dir reviews/task-124/005-tq-final-width-sweep/artifacts/final15-ab --manifest-output reviews/task-124/005-tq-final-width-sweep/artifacts/final15-ab-manifest.json --results-output reviews/task-124/005-tq-final-width-sweep/artifacts/final15-ab-results.jsonl --log-file reviews/task-124/005-tq-final-width-sweep/artifacts/final15-ab-run.log
```

## 100k Final-Width Decision Sweep

All rows use the same 100k TQ index-side sidecar shape: `rerank_width=100`, default group width 100, and scan-time final exact widths 10/15/20/25.

| final width | nprobe | recall@10 | latency p50 | latency p95 | latency p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 32 | 0.9390 | 4.81 ms | 5.32 ms | 5.68 ms |
| 10 | 64 | 0.9580 | 8.84 ms | 9.26 ms | 9.61 ms |
| 15 | 32 | 0.9730 | 4.86 ms | 5.46 ms | 5.81 ms |
| 15 | 64 | 1.0000 | 8.86 ms | 9.21 ms | 9.45 ms |
| 20 | 32 | 0.9730 | 4.92 ms | 5.50 ms | 5.91 ms |
| 20 | 64 | 1.0000 | 8.89 ms | 9.23 ms | 9.33 ms |
| 25 | 32 | 0.9730 | 4.85 ms | 5.41 ms | 5.65 ms |
| 25 | 64 | 1.0000 | 8.88 ms | 9.37 ms | 9.73 ms |

Interpretation: final10 is recall-unsafe. final15 and final20 keep the same 100k recall as final25 while reducing the exact f32 final pass from 25 source rows to 15/20 source rows. final15 was selected for the full A/B matrix because it was the lowest recall-safe 100k point in this decision sweep.

## Final15 A/B Recall

| scale | variant | nprobe32 recall@10 | nprobe64 recall@10 |
| --- | --- | ---: | ---: |
| 10k | f32/source | 1.0000 | 1.0000 |
| 10k | TQ final15 | 1.0000 | 1.0000 |
| 50k | f32/source | 0.9960 | 1.0000 |
| 50k | TQ final15 | 0.9960 | 0.9990 |
| 100k | f32/source | 0.9730 | 1.0000 |
| 100k | TQ final15 | 0.9730 | 1.0000 |

## Final15 A/B Latency

| scale | variant | nprobe32 p50 | nprobe32 p95 | nprobe32 p99 | nprobe64 p50 | nprobe64 p95 | nprobe64 p99 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 0.78 ms | 0.91 ms | 1.02 ms | 1.29 ms | 1.43 ms | 1.55 ms |
| 10k | TQ final15 | 0.74 ms | 0.86 ms | 1.10 ms | 1.17 ms | 1.33 ms | 1.46 ms |
| 50k | f32/source | 2.89 ms | 4.02 ms | 4.54 ms | 5.40 ms | 6.68 ms | 7.36 ms |
| 50k | TQ final15 | 2.42 ms | 2.68 ms | 2.89 ms | 4.90 ms | 5.16 ms | 5.34 ms |
| 100k | f32/source | 5.42 ms | 6.03 ms | 6.21 ms | 9.46 ms | 10.6 ms | 12.6 ms |
| 100k | TQ final15 | 5.23 ms | 5.73 ms | 6.01 ms | 9.87 ms | 12.9 ms | 15.0 ms |

## Final15 A/B Storage

| scale | f32/source index | TQ final15 index |
| --- | ---: | ---: |
| 10k | 2.9 MiB | 10.9 MiB |
| 50k | 11.6 MiB | 50.8 MiB |
| 100k | 22.5 MiB | 100.8 MiB |

## SIMD Counters

TQ final15 remains fully SIMD at every measured scale and nprobe:

- TQ candidates: 10,000 per latency sweep point
- TQ flushes: 100 per latency sweep point
- `scalar_candidates=0`
- `width_ge32=100`

## Interpretation

Final15 is an improvement candidate, but this packet does not close Task 124.

- Recall: final15 matches f32/source at 10k and 100k, but 50k/nprobe64 drops from 1.0000 to 0.9990. That is small, but it is not an identical recall result.
- Latency: final15 improves most p50/p95 points, especially 50k. The 100k/nprobe64 tail is worse in this same-suite run.
- Storage: TQ sidecar storage remains roughly 4x f32/source at 10k/50k/100k. This is the dominant blocker.

Next TQ optimization work should target sidecar storage/header overhead and 100k tail latency, not the SIMD scorer. The scorer is already running full SIMD in these measurements.
