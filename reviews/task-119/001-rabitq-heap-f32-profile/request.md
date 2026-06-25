---
task: 119
packet: reviews/task-119/001-rabitq-heap-f32-profile
checkpoint_sha: dd7154bb65fb7a4be2bd549dfed2d1fa71d2453a
branch: task-119-hnsw-rabitq-coarse-rerank-profile
role: coder
date: 2026-06-25
---

# Review Request: HNSW RaBitQ Coarse-Rerank M5 Profile

## Summary

Task 119 is now measured on the M5 laptop. The result is **keep experimental / iterate**, not promote.

The tested profile is true HNSW RaBitQ coarse-rerank for the current implementation surface:

- candidate generation/traversal uses `storage_format = 'rabitq'`;
- explicit overfetch uses `ef_search = 320, 500, 1000`;
- second-stage rerank uses `ec_hnsw.rerank_format = heap_f32`;
- `ec_hnsw.rerank_width = 1000` reranks the full emitted candidate pool at the decision point.

Heap-f32 rerank materially improves RaBitQ recall, but it does not create a storage win and it becomes slower at 100k. The profile is useful evidence and may justify a later iteration, but it should not be promoted as-is.

## Task 118 Gate

This packet cites Task 118 packet `reviews/task-118/006-final-attribution-matrix`.

Task 118's M5 closeout showed RaBitQ's dominant loss was candidate containment/traversal, not final exact rerank or source-vs-compressed build. That unblocked Task 119 only under a narrow go/no-go condition: measure whether a wider RaBitQ candidate frontier plus exact/source rerank can recover recall while preserving enough latency/storage advantage to matter.

Task 119's answer is mixed: the wider heap_f32 rerank recovers recall, but the current HNSW layout has no RaBitQ storage advantage and the 100k latency cost is too high.

## M5 Results

Release suite:

- Artifacts: `artifacts/suite-manifest.release2.json`, `artifacts/suite-results.release2.jsonl`, `artifacts/suite-run.release2.log`
- Status: `44/44` release suite steps succeeded.
- Backend: `artifacts/precheck-release2-extension.log` proves the benchmark DB was using a release extension before the release run.

Key `ef_search=1000` rows:

| Scale | Lane | Recall@10 | Recall mean q-time | Latency mean / p95 / p99 | Total storage | HNSW index |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| 10k | PqFastScan | 0.9965 | 5.34 ms | n/a | 172.2 MiB | 13.1 MiB |
| 10k | RaBitQ quantized | 0.9535 | 6.84 ms | 6.65 / 7.09 / 7.28 ms | 172.1 MiB | 13.0 MiB |
| 10k | RaBitQ heap_f32 w1000 | 0.9765 | 9.46 ms | 9.45 / 10.4 / 10.9 ms | 172.1 MiB | 13.0 MiB |
| 50k | PqFastScan | 0.9855 | 6.89 ms | n/a | 860.1 MiB | 65.2 MiB |
| 50k | RaBitQ quantized | 0.9380 | 8.71 ms | 8.07 / 9.09 / 9.80 ms | 860.0 MiB | 65.1 MiB |
| 50k | RaBitQ heap_f32 w1000 | 0.9885 | 11.94 ms | 12.6 / 15.1 / 16.9 ms | 860.0 MiB | 65.1 MiB |
| 100k | PqFastScan | 0.9890 | 10.44 ms | n/a | 1.7 GiB | 130.3 MiB |
| 100k | RaBitQ quantized | 0.9420 | 9.74 ms | 10.2 / 12.5 / 15.0 ms | 1.7 GiB | 130.2 MiB |
| 100k | RaBitQ heap_f32 w1000 | 0.9850 | 21.36 ms | 21.0 / 27.2 / 30.7 ms | 1.7 GiB | 130.2 MiB |

Interpretation:

- Heap-f32 rerank improves RaBitQ recall by `+0.0230` at 10k, `+0.0505` at 50k, and `+0.0430` at 100k versus quantized RaBitQ.
- At 50k, RaBitQ heap_f32 w1000 slightly beats PqFastScan recall (`0.9885` vs `0.9855`) but is slower.
- At 100k, RaBitQ heap_f32 w1000 is below PqFastScan recall (`0.9850` vs `0.9890`) and roughly doubles mean latency versus PqFastScan's recall q-time.
- Storage is effectively tied across TurboQuant, PqFastScan, and RaBitQ at each scale; the expected RaBitQ HNSW storage win is not present in this layout.

## Candidate Counters

10k full frontier diagnostic artifacts:

- `artifacts/frontier-diagnostics/frontier-10k-hnsw-rabitq-quantized.log`
- `artifacts/frontier-diagnostics/frontier-10k-hnsw-rabitq-heap-f32-w1000.log`

50k/100k counters-only artifacts:

- `artifacts/frontier-release-index-diagnostics/suite-manifest.counter20.json`
- `artifacts/frontier-release-index-diagnostics/suite-results.counter20.jsonl`
- `artifacts/frontier-release-index-diagnostics/frontier-50k-hnsw-rabitq-quantized.log`
- `artifacts/frontier-release-index-diagnostics/frontier-50k-hnsw-rabitq-heap-f32-w1000.log`
- `artifacts/frontier-release-index-diagnostics/frontier-100k-hnsw-rabitq-quantized.log`
- `artifacts/frontier-release-index-diagnostics/frontier-100k-hnsw-rabitq-heap-f32-w1000.log`

At `ef_search=1000`:

| Scale | Mode | Queries | Emitted pool | Exact rerank | Quantized rerank | Dropped before exact |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | quantized | 200 | 1000 | 0 | 1000 | 1000 |
| 10k | heap_f32 w1000 | 200 | 1000 | 1000 | 0 | 0 |
| 50k | quantized | 20 | 1000 | 0 | 1000 | 1000 |
| 50k | heap_f32 w1000 | 20 | 1000 | 1000 | 0 | 0 |
| 100k | quantized | 20 | 1000 | 0 | 1000 | 1000 |
| 100k | heap_f32 w1000 | 20 | 1000 | 1000 | 0 | 0 |

The counters prove the measured heap_f32 lane is not just default RaBitQ scoring: it reranks the emitted pool with exact heap/source scores before final output.

## Code Checkpoint

This packet includes commit `dd7154bb6`:

- adds a pg_test counter-only HNSW frontier export that skips full corpus ground-truth work;
- adds `ecaz bench hnsw-frontier --counters-only`;
- lets `ecaz bench suite` expand `counters_only: true`;
- updates the Task 119 release-index frontier reuse suite to use 20-query counters-only probes.

Validation artifacts:

- `artifacts/cargo-check-ecaz-cli-counter-probe.log`
- `artifacts/cargo-check-ecaz-pg18-pgtest-counter-probe.log`
- `artifacts/cargo-test-ecaz-cli-hnsw-frontier-counter-probe.log`
- `artifacts/suite-audit-frontier-reuse-counter20.log`

## Recommendation

Keep HNSW RaBitQ coarse-rerank experimental and iterate only if a follow-up changes the storage or candidate-generation economics.

Concrete next direction:

- do not promote `rabitq + heap_f32 w1000` as-is;
- investigate why HNSW RaBitQ has no index-size win in the current layout before spending more time on productionizing this profile;
- if continuing, test a profile that stores materially less per node or uses a cheaper stronger rerank than heap_f32 at 100k+.

No storage format or on-disk layout change landed in this packet.
