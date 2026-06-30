# Task 124 Packet 018: Selected Payload Vector Index Negative Result

## Summary

This packet follows the corrected Task 124 objective from packet `017`: improve
TurboQuant speed and report TQ-before/TQ-after evidence.

I tested a narrow TQ materialization overhead change and rejected it. The
temporary code replaced the selected-payload slab's per-group
`HashMap<ItemPointer, usize>` lookup with a compact vector index. The hypothesis
was that selected sets are small enough that avoiding the hash map allocation and
hash probes would reduce TQ stage-2 overhead.

It did not. The 100k TQ speed suite regressed latency at both nprobe points, so
the code was reverted. There is no source change proposed for landing from this
packet.

Temporary diff:

- `artifacts/discarded-selected-index-vector.diff`

## Evidence

Artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-selected-index-vector-100k-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/selected-index-vector-100k/latency-100k-tq-w75-g50-final15-vector.log`
- `artifacts/selected-index-vector-100k/recall-100k-tq-w75-g50-final15-vector.log`
- `artifacts/selected-index-vector-100k/storage-100k-tq-w75-g50-final15-vector.log`

Suite status:

```text
[suite:task124-tq-selected-index-vector-100k-suite] completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
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

Latency regressed versus packet `011`:

| nprobe | packet 011 p50/p95/p99 | packet 018 p50/p95/p99 |
| ---: | ---: | ---: |
| 32 | 4.83 / 5.35 / 5.55 ms | 5.03 / 5.59 / 5.93 ms |
| 64 | 8.91 / 9.14 / 9.25 ms | 9.37 / 9.66 / 9.77 ms |

TQ scoring remained full SIMD:

| nprobe | quant | isa | scalar candidates | candidates |
| ---: | --- | --- | ---: | ---: |
| 32 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

## Decision

Do not land this vector-index selected-payload lookup change. It is a measured
TQ speed regression.

Task 124 remains open under the corrected speed objective. The next useful TQ
speed slice should target a different hot-path cost, not this inner selected
lookup representation.
