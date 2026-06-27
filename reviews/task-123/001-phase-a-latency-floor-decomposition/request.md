# Task 123 Phase A Gate: Flat Floor vs SPIRE Scan Cost

## Summary

This packet asks for review of the Task 123 Phase A gate evidence. No code changed in this slice; it is a measurement packet using existing Task 121 Phase 3 `b4/tr50/f8` SPIRE surfaces at 10k / 50k / 100k.

The result is a no-go for proceeding directly into the Phase B `nlists x boundary` factorial unless a reviewer wants to override the gate. SPIRE recovers recall at nprobe 96, but the recall-1.0 path is well outside the 5-10x flat-floor envelope:

| Scale | Flat exact p50 | SPIRE nprobe 96 p50 | Ratio | Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 29.4 ms | 496.2 ms | 16.9x | 1.0000 |
| 50k | 80.2 ms | 2159.5 ms | 26.9x | 1.0000 |
| 100k | 223.3 ms | 5483.0 ms | 24.6x | 1.0000 |

At the lower operating point, nprobe 8 is closer to the flat floor but does not preserve recall across scales:

| Scale | SPIRE nprobe 8 p50 | Ratio vs flat | Recall@10 |
| --- | ---: | ---: | ---: |
| 10k | 103.8 ms | 3.5x | 0.9875 |
| 50k | 428.2 ms | 5.3x | 0.9938 |
| 100k | 965.9 ms | 4.3x | 0.9375 |

## Evidence

- Suite config: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/task123-phase-a-suite.json`
- Suite manifest/results: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/suite-manifest.json`, `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/suite-results.jsonl`
- Artifact manifest: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/manifest.md`
- Flat-floor plan proof: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/flat-floor-plan.log`
- Latency logs: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/latency-flat-floor-*.log`, `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/latency-spire-*-nprobe-8-96.log`
- Pipeline decomposition: `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/spire-pipeline-*-nprobe-8-96.log`, `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/funnel-*-nprobe-8-96.jsonl`, `reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/stage-containment-*-nprobe-8-96.jsonl`

## Finding

The binding wall is the local scan/candidate path, not route precision alone.

Route-stage containment equals final recall in every measured row. At nprobe 96, all scales are 320/320 truth-contained at the route stage and 320/320 at final top-k. At nprobe 8, route containment and final recall match at 316/320 for 10k, 318/320 for 50k, and 300/320 for 100k.

The high-recall path is expensive because it scans and scores too much local-store data:

| Scale | nprobe | Candidates/query | Object bytes/query | Leaf read ms/query | Candidate score ms/query | Heap append ms/query |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 96 | 37,861 | 30.4 MiB | 10.5 | 19.5 | 11.0 |
| 50k | 96 | 186,824 | 149.8 MiB | 96.2 | 95.4 | 71.2 |
| 100k | 96 | 378,986 | 303.7 MiB | 210.2 | 199.0 | 164.0 |

The 100k nprobe 96 row reads roughly 303.7 MiB/query from local-store objects for a 223.3 ms flat exact floor, then reports SPIRE p50 around 5.5 s in the clean latency run. That is enough to fail the Phase A gate before spending time on Phase B.

## Recommendation

Treat Task 123 Phase A as a re-scope point. Do not run the full `nlists {316,512,1024} x boundary {0,1,2}` matrix until the scan path has a credible fix or an explicit reviewer decision says the factorial is still worth measuring. The owning follow-up should point at the IVF/SPIRE scan-efficiency line and the SPIRE distributed/local-store transport path, with this packet as the flat-floor evidence.
