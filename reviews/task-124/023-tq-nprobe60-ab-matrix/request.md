# Task 124 Packet 023: TQ Nprobe 60 A/B Matrix

## Summary

Packet `021` and `022` identified the coarse frontier as the remaining latency
lever for the current TQ shape. This packet carries the candidate `nprobe=60`
setting through the required 10k/50k/100k matrix against the current
`nprobe=64` high-recall reference.

Result: `nprobe=60` preserves the `nprobe=64` recall result at every measured
scale in this run and improves p50/p95/p99 latency at every measured scale.
Storage does not change.

This is a TQ speed improvement by frontier reduction, not a fix for the TQ
storage wall.

## Evidence

Artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-nprobe60-ab-10-50-100-suite.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-run.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/report-results.jsonl`
- `artifacts/nprobe60-ab-matrix/*-10k-*.log`
- `artifacts/nprobe60-ab-matrix/*-50k-*.log`
- `artifacts/nprobe60-ab-matrix/*-100k-*.log`

Suite status:

```text
[suite:task124-tq-nprobe60-ab-10-50-100-suite] completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

TQ shape:

- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`
- `nprobe=60` compared against `nprobe=64`

Recall and latency:

| scale | nprobe | recall@k | ndcg@k | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 60 | 1.0000 | 1.0000 | 1.13 ms | 1.22 ms | 1.29 ms |
| 10k | 64 | 1.0000 | 1.0000 | 1.20 ms | 1.32 ms | 1.41 ms |
| 50k | 60 | 0.9980 | 1.0000 | 4.15 ms | 4.50 ms | 4.99 ms |
| 50k | 64 | 0.9980 | 1.0000 | 4.58 ms | 4.83 ms | 5.08 ms |
| 100k | 60 | 1.0000 | 1.0000 | 8.65 ms | 9.01 ms | 9.35 ms |
| 100k | 64 | 1.0000 | 1.0000 | 8.98 ms | 9.36 ms | 9.92 ms |

Latency improvement for `nprobe=60`:

| scale | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| 10k | 0.07 ms faster | 0.10 ms faster | 0.12 ms faster |
| 50k | 0.43 ms faster | 0.33 ms faster | 0.09 ms faster |
| 100k | 0.33 ms faster | 0.35 ms faster | 0.57 ms faster |

The TQ scorer remains full SIMD:

| scale | nprobe | isa | scalar candidates | TQ candidates |
| --- | ---: | --- | ---: | ---: |
| 10k | 60 | neon | 0 | 7500 |
| 10k | 64 | neon | 0 | 7500 |
| 50k | 60 | neon | 0 | 7500 |
| 50k | 64 | neon | 0 | 7500 |
| 100k | 60 | neon | 0 | 7500 |
| 100k | 64 | neon | 0 | 7500 |

The measured speedup is attributable to less coarse RaBitQ frontier work, not a
change in the fixed TQ stage-2 scorer:

| scale | coarse candidates at nprobe 60 | coarse candidates at nprobe 64 |
| --- | ---: | ---: |
| 10k | 936366 | 1000000 |
| 50k | 4525933 | 5000000 |
| 100k | 9556278 | 10000000 |

Storage remains unchanged:

| scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 10.9 MiB | 1143.6 B |
| 50k | 50.9 MiB | 1066.8 B |
| 100k | 100.8 MiB | 1057.2 B |

## Interpretation

`nprobe=60` is a valid measured TQ latency improvement over the current
`nprobe=64` reference for this TQ shape. It preserves the observed recall result
at 10k, 50k, and 100k while reducing p50/p95/p99 latency across the matrix.

This does not resolve the broader TQ competitiveness issue. The TQ scorer was
already SIMD-only, and the persistent TQ sidecar remains the same size. The win
comes from trimming the coarse candidate frontier before TQ stage 2, which
reduces RaBitQ candidate work while leaving TQ stage-2 work fixed at 7,500
candidates.

## Decision

Keep `nprobe=60` as the current TQ speed candidate. Do not claim a storage
improvement. Further closeout still needs a reviewer decision on whether this
latency-only TQ improvement is enough for Task 124, given the unchanged storage
wall and the prior task guidance against non-structural churn.

