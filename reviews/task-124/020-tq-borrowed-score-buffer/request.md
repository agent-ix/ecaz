# Task 124 Packet 020: Borrowed Score Buffer Negative Result

## Summary

This packet continues the corrected Task 124 objective from packet `017`:
improve TurboQuant speed and report TQ-before/TQ-after evidence.

I tested a narrow TQ stage-2 scoring overhead change and rejected it. The
temporary code added a TurboQuant borrowed-payload batch scorer that writes
directly into the caller's score buffer, avoiding the extra temporary
`estimates` vector and copy/negate loop in the index-side TQ rerank path.

It did not improve end-to-end latency. The 100k TQ suite was slightly worse
than the packet `011` selected-slab baseline at both nprobe points, so the code
was reverted. There is no source change proposed for landing from this packet.

Temporary diff:

- `artifacts/discarded-borrowed-score-buffer.diff`

## Evidence

Artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-borrowed-score-buffer-100k-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/borrowed-score-buffer-100k/latency-100k-tq-w75-g50-final15-borrowed-score-buffer.log`
- `artifacts/borrowed-score-buffer-100k/recall-100k-tq-w75-g50-final15-borrowed-score-buffer.log`
- `artifacts/borrowed-score-buffer-100k/storage-100k-tq-w75-g50-final15-borrowed-score-buffer.log`

Suite status:

```text
[suite:task124-tq-borrowed-score-buffer-100k-suite] completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

Same measured TQ shape as packet `011`:

- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`
- 100k staged corpus

Recall stayed unchanged:

| nprobe | recall@k | ndcg@k |
| ---: | ---: | ---: |
| 32 | 0.9730 | 0.9969 |
| 64 | 1.0000 | 1.0000 |

Latency did not improve versus packet `011`:

| nprobe | packet 011 p50/p95/p99 | packet 020 p50/p95/p99 |
| ---: | ---: | ---: |
| 32 | 4.83 / 5.35 / 5.55 ms | 4.86 / 5.41 / 5.76 ms |
| 64 | 8.91 / 9.14 / 9.25 ms | 9.05 / 9.48 / 9.60 ms |

TQ scoring remained full SIMD:

| nprobe | quant | isa | scalar candidates | candidates |
| ---: | --- | --- | ---: | ---: |
| 32 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

## Decision

Do not land the borrowed-score-buffer change. It removes a small allocation on
paper, but the measured end-to-end TQ path did not get faster.

Task 124 remains open under the corrected speed objective. The next useful TQ
speed slice should target a larger cost center than the final score-buffer
handoff, such as reducing sidecar group/header work, cutting payload bytes
touched, or changing the stage-2/final-rerank boundary.
