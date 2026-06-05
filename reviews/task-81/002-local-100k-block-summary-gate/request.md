# Task 81 Packet 002: Local 100k Block-Summary Gate

## Request

Review the local PG18 100k RaBitQ gate for Task 81 block-summary pruning. This is a measurement-only packet on top of the accepted diagnostics/ADR slice in packet 001.

The packet compares old full-leaf behavior against block-summary pruning using `ecaz bench suite`, with matched routing and rerank settings:

- shared existing fixture: `task79_spire_candidate_surface`
- corpus: `task79_surface_100k_corpus`
- queries: `task79_surface_100k_queries`
- index: `task79_surface_100k_idx`
- `nprobe=96`
- `rerank_width=25`
- 200 queries
- storage format `rabitq`

## Result

The local gate clears.

| Row | Candidates | p50 | p95 | p99 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| full leaf, nprobe 96 | 15,506,227 | 65.106 ms | 77.548 ms | 97.520 ms | 0.9975 |
| block summary, global 1152 | 3,673,383 | 33.472 ms | 40.328 ms | 46.322 ms | 0.9940 |

This cuts candidate scoring by 76.31% while staying above the Task 81 recall floor (`0.9940 >= 0.9925`) and below the local p50 gate (`33.472 ms <= 45 ms`).

The diagnostic aggregate over the same 200-query row confirms the pruning attribution:

- `blocks_available=977202`
- `blocks_selected=230400`
- `blocks_skipped=746802`
- `summary_score_nanos=1282625876`
- `row_score_nanos=1636663557`
- `summary_bytes=2323638996`
- `row_bytes=12641349328`

## Evidence

- Artifact manifest: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/manifest.md`
- Suite config: `reviews/task-81/002-local-100k-block-summary-gate/suite-local-100k-block-summary-gate.json`
- Suite manifest: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-manifest.json`
- Status: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-status.log`
- Report: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-report.log`
- Raw comparator logs:
  - `reviews/task-81/002-local-100k-block-summary-gate/artifacts/pipeline-100k-rabitq-full-leaf-nprobe96.log`
  - `reviews/task-81/002-local-100k-block-summary-gate/artifacts/pipeline-100k-rabitq-block-summary-global1152.log`
  - `reviews/task-81/002-local-100k-block-summary-gate/artifacts/diagnostics-100k-rabitq-block-summary-global1152.log`

## Reviewer Focus

1. Check that the suite config keeps routing/rerank comparable between the full-leaf and block-summary rows.
2. Check that the local result satisfies the Task 81 local gates and is suitable to unlock the AWS 1M follow-up.
3. Check that the diagnostic aggregate is consistent with the pipeline candidate count and demonstrates selected/skipped block accounting.
