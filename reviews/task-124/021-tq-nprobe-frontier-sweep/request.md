# Task 124 Packet 021: TQ Nprobe Frontier Sweep

## Summary

This packet tests a TurboQuant-specific speed lever after packets `018` and
`020` showed that small stage-2 micro-optimizations did not move latency.

The measured bottleneck is the coarse RaBitQ frontier work, not the TQ stage-2
SIMD scorer: TQ stage-2 remains around 7,500 candidates and under 2 ms total
kernel time across the 100-query latency run, while coarse RaBitQ scans millions
of candidates. This packet therefore tests whether the current best TQ shape can
use a lower `nprobe` frontier while preserving recall.

This is diagnostic evidence, not Task 124 closeout. It reuses the isolated 100k
index from packet `020`, after reinstalling the reverted branch source so the
discarded packet `020` code was not under test.

## Evidence

Artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-nprobe-frontier-100k-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/nprobe-frontier-100k/recall-100k-tq-w75-g50-final15-nprobe-frontier.log`
- `artifacts/nprobe-frontier-100k/latency-100k-tq-w75-g50-final15-nprobe-frontier.log`
- `artifacts/nprobe-frontier-100k/storage-100k-tq-w75-g50-final15-nprobe-frontier.log`

Suite status:

```text
[suite:task124-tq-nprobe-frontier-100k-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
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
| 32 | 0.9730 | 0.9969 | 4.83 ms | 5.49 ms | 5.80 ms |
| 40 | 0.9910 | 0.9992 | 5.94 ms | 6.48 ms | 6.67 ms |
| 48 | 0.9940 | 0.9996 | 7.09 ms | 7.66 ms | 8.07 ms |
| 56 | 0.9990 | 1.0000 | 8.03 ms | 8.34 ms | 8.63 ms |
| 64 | 1.0000 | 1.0000 | 8.88 ms | 9.12 ms | 9.34 ms |

TQ scoring remained full SIMD at every nprobe:

| nprobe | quant | isa | scalar candidates | candidates |
| ---: | --- | --- | ---: | ---: |
| 32 | turboquant | neon | 0 | 7500 |
| 40 | turboquant | neon | 0 | 7500 |
| 48 | turboquant | neon | 0 | 7500 |
| 56 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

Storage is unchanged from the current best TQ shape:

- `ec_ivf index=100.8 MiB`
- `per_row=1057.2 B`

## Interpretation

This is the first measured Task 124 result that points to a meaningful speed
path after the stage-2 micro-optimization failures:

- `nprobe=56` cuts p50/p95/p99 latency versus `nprobe=64` by about
  `0.85 / 0.78 / 0.71 ms` at 100k.
- Recall drops from `1.0000` to `0.9990`, with `ndcg@k=1.0000`.
- `nprobe=48` and below are faster but lose more recall.

This does not prove completion yet. The next step is to broaden the promising
`nprobe=56` vs `64` comparison to 10k/50k/100k before deciding whether this is
an acceptable TQ speed configuration.

## Decision

Keep Task 124 open. Continue with a 10k/50k/100k TQ matrix focused on the
`nprobe=56` frontier-reduction candidate against the current `nprobe=64`
high-recall reference.
