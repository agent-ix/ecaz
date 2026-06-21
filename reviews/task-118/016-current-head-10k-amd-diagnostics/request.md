---
task: 118
packet: reviews/task-118/016-current-head-10k-amd-diagnostics
checkpoint_sha: 5ff394624d8dc8e465919f28bd78f3f0e622ab4c
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Current-Head 10k AMD Diagnostics

## Scope

This packet regenerates the 10k Task 118 HNSW frontier and score-correlation
diagnostics on the current branch head after the candidate-pool diagnostic fix
and the ef=200 suite narrowing.

This was run on the slower AMD host, so it is not a substitute for the final
Intel 50k/100k closeout evidence. It is still useful current-head 10k evidence:
the frontier diagnostic now reports the AM's ef_search-sized candidate pool
instead of the previously empty pre-output frontier rows, and the source-build
vs compressed-build 10k rows remain identical.

## Validation

- 10k frontier current-head AMD suite:
  - Artifact: `artifacts/suite-run-10k-frontier-current-head-amd.log`
  - Artifact: `artifacts/suite-manifest-10k-frontier-current-head-amd.json`
  - Artifact: `artifacts/results-10k-frontier-current-head-amd.jsonl`
- 10k score-correlation current-head AMD suite:
  - Artifact: `artifacts/suite-run-10k-score-current-head-amd.log`
  - Artifact: `artifacts/suite-manifest-10k-score-current-head-amd.json`
  - Artifact: `artifacts/results-10k-score-current-head-amd.jsonl`

Raw per-query diagnostic JSONL files were generated locally by the diagnostic
commands but are intentionally not part of this review packet.

## Key Rows

At `ef_search=200`, source-build and compressed-build frontier containment
match:

| lane | truth@10 in frontier | truth@100 in frontier | frontier | exact rerank | dropped before exact |
| --- | ---: | ---: | ---: | ---: | ---: |
| TurboQuant source | 0.9965 | 0.9545 | 200.0 | 200.0 | 0.0 |
| PqFastScan source | 0.9960 | 0.9543 | 200.0 | 200.0 | 0.0 |
| RaBitQ source | 0.9705 | 0.9272 | 200.0 | 200.0 | 0.0 |
| TurboQuant compressed-build | 0.9965 | 0.9545 | 200.0 | 200.0 | 0.0 |
| PqFastScan compressed-build | 0.9960 | 0.9543 | 200.0 | 200.0 | 0.0 |
| RaBitQ compressed-build | 0.9705 | 0.9272 | 200.0 | 200.0 | 0.0 |

Score-correlation also matches source-build vs compressed-build:

| lane | mean Spearman | mean abs rank shift | max abs rank shift | exact best approx rank | exact top4 max approx rank | missing cmp |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| TurboQuant source | 0.8404 | 22.79 | 175 | 1.4 | 6.1 | 0.0 |
| PqFastScan source | 0.8404 | 22.78 | 175 | 1.4 | 6.1 | 0.0 |
| RaBitQ source | 0.9086 | 16.85 | 165 | 1.3 | 4.9 | 0.0 |
| TurboQuant compressed-build | 0.8404 | 22.79 | 175 | 1.4 | 6.1 | 0.0 |
| PqFastScan compressed-build | 0.8404 | 22.78 | 175 | 1.4 | 6.1 | 0.0 |
| RaBitQ compressed-build | 0.9086 | 16.85 | 165 | 1.3 | 4.9 | 0.0 |

The 10k current-head result keeps the dominant local diagnosis unchanged:
RaBitQ's loss is visible in candidate containment (`truth@10 in frontier =
0.9705`) while exact rerank covers the full 200-candidate frontier and score
correlation is stronger than TurboQuant/PqFastScan.

## Remaining Task 118 Closeout Work

Run the Intel 50k/100k closeout suites from the current branch head and update
packet 006 with the final dominant-loss classification. This AMD-local packet
should not be treated as final host-class performance evidence.
