# Task 123 Final Closeout Request

## Scope

This is a synthesis-only closeout request for Task 123. It adds no new
benchmark run. It asks the reviewer to sign off that packet
`001-phase-a-latency-floor-decomposition` is sufficient to close Task 123 at
the Phase A gate, with Phase B/C deferred.

## Verdict Requested

Close Task 123 as an evidence-backed Phase A no-go / re-scope result.

Task 123 asked whether SPIRE route precision can be recovered at competitive
absolute cost, or whether the scan path itself is the binding wall. The Phase A
gate answered that question before Phase B:

- SPIRE recovers recall at nprobe 96, but the recall-1.0 path is outside the
  task's 5-10x flat-floor envelope at all three scales.
- Route-stage containment equals final recall at every measured nprobe/scale,
  so the remaining wall is not candidate-stage or rerank recall loss.
- The concrete wall is the post-route local scan/candidate path: high-recall
  100k scans about 379k candidates/query and reads about 303.7 MiB/query from
  local-store objects.

## Evidence Summary

Primary evidence packet:
`reviews/task-123/001-phase-a-latency-floor-decomposition/`

Key packet-local files:

- `artifacts/manifest.md`
- `artifacts/task123-phase-a-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-results.jsonl`
- `artifacts/flat-floor-plan.log`
- `artifacts/latency-flat-floor-*.log`
- `artifacts/latency-spire-*-nprobe-8-96.log`
- `artifacts/spire-pipeline-*-nprobe-8-96.log`
- `artifacts/funnel-*-nprobe-8-96.jsonl`
- `artifacts/stage-containment-*-nprobe-8-96.jsonl`

Task status sync:
`reviews/task-123/002-phase-a-status-sync/`

## Acceptance-Criteria Audit

### AC1: Phase A floor + SPIRE decomposition

Satisfied by packet 001.

Flat exact floors were measured at 10k / 50k / 100k with index scans disabled
and sequential scans proved in `flat-floor-plan.log`.

Clean latency from `suite-results.jsonl`:

| Scale | Flat exact p50 | Flat exact p95 | SPIRE nprobe 96 p50 | SPIRE nprobe 96 p95 | Ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 29.4 ms | 51.6 ms | 496.2 ms | 560.0 ms | 16.9x |
| 50k | 80.2 ms | 168.7 ms | 2159.5 ms | 2634.7 ms | 26.9x |
| 100k | 223.3 ms | 354.3 ms | 5483.0 ms | 6233.7 ms | 24.6x |

Per-stage decomposition is in `spire-pipeline-*-nprobe-8-96.log`,
`funnel-*-nprobe-8-96.jsonl`, and
`stage-containment-*-nprobe-8-96.jsonl`.

Binding wall named: post-route local scan/candidate path.

### AC2: Phase B nlists x boundary factorial

Deferred by the explicit Phase A gate.

Task 123 says Phase A decides whether Phase B/C are worth running. The measured
recall-1.0 path misses the 5-10x flat-floor gate at every scale, so running the
full `nlists x boundary` factorial would optimize route precision after the
task's own gate has already shown high-recall SPIRE is not competitive on
absolute scan cost.

The status sync in packet 002 records this in the canonical task file rather
than silently dropping Phase B.

### AC3: Decisive verdict

Satisfied as the second allowed outcome: evidence-backed proof that the scan
path is the wall.

At nprobe 96, route containment and final recall are both 100% at every scale.
At nprobe 8, route containment and final recall also match:

| Scale | nprobe | Route containment | Final recall basis |
| --- | ---: | ---: | ---: |
| 10k | 8 | 316 / 320 = 98.75% | 316 / 320 = 98.75% |
| 10k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |
| 50k | 8 | 318 / 320 = 99.375% | 318 / 320 = 99.375% |
| 50k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |
| 100k | 8 | 300 / 320 = 93.75% | 300 / 320 = 93.75% |
| 100k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |

The owning follow-up direction is the IVF/SPIRE scan-efficiency line, especially
candidate-frontier and scan-locality architecture in Tasks 111/111e, plus a
SPIRE-specific local-store/transport-efficiency follow-up if SPIRE continues as
a local-store AM.

### AC4: Finding-tied recommendations

Satisfied by packet 001's route-stage funnel and flat-floor comparison.

The recommendation does not rely on aggregate final recall alone:

- route containment is traced through `stage-containment-*.jsonl`;
- flat floor is traced through `flat-floor-plan.log` and
  `latency-flat-floor-*.log`;
- scan/candidate volume is traced through `spire-pipeline-*.log` and
  `funnel-*.jsonl`;
- the status update links the recommendation back to those packet-local
  artifacts.

## Requested Reviewer Decision

Please either:

1. sign off on closing Task 123 as a Phase A no-go / re-scope result, or
2. explicitly request Phase B despite the failed gate and state what evidence
   would make that factorial worth running.
