# Task 81 AWS 1M Nprobe128 Block-Summary Gate

## Request

Review the AWS 1M follow-up to packet 004's local nprobe sweep. Packet 004 showed that `nprobe=128` improved local recall from `0.9945` to `0.9965` with essentially flat candidates. This packet tests whether that route-breadth change closes the Task 81 AWS gate on the retained 1M surface.

Measured shape:

- retained AWS profile `1m`
- retained q500 corpus/query lane: `task67_1m_hnsw_m7g2xlarge_*`
- retained tg256 block-summary index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- `nprobe=128`
- `rerank_width=25`
- `ec_spire.leaf_block_pruning_max_global_blocks=1152`
- truth cache: `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`

## Result

The AWS gate still does **not** pass.

| Row | Candidates | p50 | p95 | p99 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| old tg96 comparator | 9,213,846 | 268.824 ms | 331.460 ms | 345.762 ms | 0.9832 |
| packet 003 nprobe96 global1152 | 9,213,846 | 265.911 ms | 329.407 ms | 342.454 ms | 0.9832 |
| packet 005 nprobe128 global1152 | 9,213,838 | 303.107 ms | 390.202 ms | 408.786 ms | 0.9832 |

`nprobe=128` preserves the candidate-surface gate by staying 8 rows below the old q500 surface, but recall remains unchanged at `0.9832`, and latency regresses. This means packet 004's local recall gain does not transfer to the AWS retained 1M surface.

Diagnostic aggregate:

- `blocks_available=30,966,000`
- `blocks_selected=576,000`
- `blocks_skipped=30,390,000`
- `summary_score_nanos=31,464,838,658`
- `row_score_nanos=5,068,876,571`
- `candidate_score_nanos=36,533,715,229`

The selected block count remains exactly `1152 * 500`, so the global cap is applied as intended.

## Decision

Task 81 remains active. The accepted local path is still valid evidence, but AWS 1M has now rejected both candidate-preserving rows tested so far:

- packet 003: `nprobe=96`, recall unchanged
- packet 005: `nprobe=128`, recall unchanged and latency worse

## Evidence

- `artifacts/manifest.md`
- `suite-aws-1m-nprobe128-block-summary-gate.json`
- `artifacts/suite-audit.log`
- `artifacts/aws-1m-nprobe128-block-summary-gate/suite-manifest.json`
- `artifacts/aws-1m-nprobe128-block-summary-gate/results.jsonl`
- `artifacts/aws-1m-nprobe128-block-summary-gate/suite-status.log`
- `artifacts/aws-1m-nprobe128-block-summary-gate/suite-report.log`
- `artifacts/aws-1m-nprobe128-block-summary-gate/suite-report-results.jsonl`
- `artifacts/aws-1m-nprobe128-block-summary-gate/pipeline-spire-1m-rabitq-block-summary-global1152-nprobe128.log`
- `artifacts/aws-1m-nprobe128-block-summary-gate/diagnostics-spire-1m-rabitq-block-summary-global1152-nprobe128.log`
- `artifacts/cloud-status-after-pause-final-paused.log`

## Reviewer Focus

1. Confirm that this packet correctly fails the AWS Task 81 gate because recall did not improve over `0.9832`.
2. Confirm the candidate-surface comparison is still valid despite the 8-row decrease.
3. Confirm the latency regression means nprobe128 should not be promoted as the Task 81 AWS row.
