# Task 124 Packet 022: TQ Nprobe Frontier Refine

## Summary

Packet `021` found that lowering the coarse frontier is the first meaningful TQ
speed lever after several stage-2 micro-optimizations failed. It also showed
that `nprobe=56` was faster than `64` but lost a small amount of recall.

This packet refines the promising range. Result: `nprobe=60` preserves the
100k `nprobe=64` recall result (`1.0000`) while reducing latency.

This is still not Task 124 closeout. It is the candidate setting to carry into a
10k/50k/100k matrix.

## Evidence

Artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-nprobe-refine-100k-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/nprobe-refine-100k/recall-100k-tq-w75-g50-final15-nprobe-refine.log`
- `artifacts/nprobe-refine-100k/latency-100k-tq-w75-g50-final15-nprobe-refine.log`
- `artifacts/nprobe-refine-100k/storage-100k-tq-w75-g50-final15-nprobe-refine.log`

Suite status:

```text
[suite:task124-tq-nprobe-refine-100k-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

TQ shape:

- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`
- 100k staged corpus

Recall and latency:

| nprobe | recall@k | ndcg@k | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 56 | 0.9990 | 1.0000 | 8.26 ms | 8.57 ms | 9.31 ms |
| 58 | 0.9990 | 1.0000 | 8.24 ms | 8.59 ms | 8.75 ms |
| 60 | 1.0000 | 1.0000 | 8.44 ms | 8.88 ms | 9.16 ms |
| 62 | 1.0000 | 1.0000 | 9.07 ms | 9.28 ms | 9.51 ms |
| 64 | 1.0000 | 1.0000 | 9.24 ms | 9.52 ms | 9.71 ms |

TQ scoring remained full SIMD:

| nprobe | quant | isa | scalar candidates | candidates |
| ---: | --- | --- | ---: | ---: |
| 56 | turboquant | neon | 0 | 7500 |
| 58 | turboquant | neon | 0 | 7500 |
| 60 | turboquant | neon | 0 | 7500 |
| 62 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

Storage is unchanged:

- `ec_ivf index=100.8 MiB`
- `per_row=1057.2 B`

## Interpretation

`nprobe=60` is the best refined candidate:

- matches `nprobe=64` recall and NDCG in this 100k run;
- improves p50 by `0.80 ms`;
- improves p95 by `0.64 ms`;
- improves p99 by `0.55 ms`;
- keeps TQ scoring on full NEON/SIMD with zero scalar candidates.

The speed win comes from cutting coarse RaBitQ frontier work, not from changing
the TQ stage-2 scorer. That is aligned with the measured counters: TQ stage-2
stays fixed at 7,500 candidates while RaBitQ candidate work falls with nprobe.

## Decision

Carry `nprobe=60` into the required 10k/50k/100k matrix against the current
`nprobe=64` high-recall reference. Do not close Task 124 from this 100k-only
packet.
