# Task 124 Packet 009 Artifact Manifest

- Task bucket: `reviews/task-124/`
- Packet path: `reviews/task-124/009-tq-stage2-width-sweep/`
- Head SHA: `676f924e4b377c833949285ed98f450e3cf9e464`
- Timestamp: `2026-06-29T03:53:24Z`
- Lane: local PG18, release extension install from prior packet, `ec_ivf` staged real 100k corpus
- Storage format: `coarse_rerank`
- Coarse frontier: `coarse_format=rabitq`, `coarse_bits=1`, `nlists=64`, recall/latency sweeps at `nprobe=32,64`
- Stage-2 variant: `rerank_placement=index`, `rerank_format=turboquant`, `stage2_final_rerank_width=15`
- Stage-2 width sweep: `rerank_width=25,50,75`
- Fixture isolation: one index/table prefix per width: `task124_tq_w25_100k`, `task124_tq_w50_100k`, `task124_tq_w75_100k`
- Outcome: diagnostic only; width 75 preserves 100k recall and remains full SIMD, but latency gain is modest and storage remains far above the f32/source baseline.

## Code Under Test

No code change was introduced for this packet.

Commit under test:

- `676f924e4b377c833949285ed98f450e3cf9e464`

This commit includes the earlier Task 124 experimental TQ variants, but this packet exercises the existing full 4-bit `turboquant` rerank format only.

## Suite Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/009-tq-stage2-width-sweep/artifacts/task124-tq-stage2-width-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/009-tq-stage2-width-sweep/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/009-tq-stage2-width-sweep/artifacts/stage2-width-100k/suite-manifest.json
```

Report summary: `completed 12`, `failed 0`, `skipped 0`.

## Committed Artifacts

- `task124-tq-stage2-width-100k-suite.json`: SuiteConfig for the 100k width sweep.
- `suite-run.log`: suite runner log.
- `stage2-width-100k/suite-manifest.json`: structured suite manifest.
- `stage2-width-100k/results.jsonl`: structured result records.
- `stage2-width-100k/load-*.log`: per-width load logs.
- `stage2-width-100k/recall-*.log`: per-width recall logs.
- `stage2-width-100k/latency-*.log`: per-width latency logs with candidate-batch counters.
- `stage2-width-100k/storage-*.log`: per-width storage logs.

Regenerable `truth-*.json` caches were intentionally not committed.

## Key Results

### Recall at k=10

| Stage-2 width | nprobe=32 | nprobe=64 |
| ---: | ---: | ---: |
| 25 | 0.9530 | 0.9790 |
| 50 | 0.9710 | 0.9980 |
| 75 | 0.9730 | 1.0000 |

### Latency

| Stage-2 width | nprobe | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: |
| 25 | 32 | 4.58 ms | 5.10 ms | 5.43 ms |
| 25 | 64 | 8.59 ms | 8.86 ms | 9.20 ms |
| 50 | 32 | 4.78 ms | 5.32 ms | 5.60 ms |
| 50 | 64 | 8.97 ms | 10.5 ms | 10.9 ms |
| 75 | 32 | 4.86 ms | 5.46 ms | 5.63 ms |
| 75 | 64 | 9.07 ms | 9.41 ms | 9.72 ms |

### Storage

| Stage-2 width | ec_ivf index size | per row |
| ---: | ---: | ---: |
| 25 | 116.3 MiB | 1219.1 B |
| 50 | 100.8 MiB | 1057.2 B |
| 75 | 105.9 MiB | 1110.0 B |

### Kernel Counters

| Stage-2 width | nprobe | quant | isa | scalar_candidates | TQ candidates | width bucket |
| ---: | ---: | --- | --- | ---: | ---: | --- |
| 25 | 32 | turboquant | neon | 0 | 2500 | `width_16_31=100` |
| 25 | 64 | turboquant | neon | 0 | 2500 | `width_16_31=100` |
| 50 | 32 | turboquant | neon | 0 | 5000 | `width_ge32=100` |
| 50 | 64 | turboquant | neon | 0 | 5000 | `width_ge32=100` |
| 75 | 32 | turboquant | neon | 0 | 7500 | `width_ge32=100` |
| 75 | 64 | turboquant | neon | 0 | 7500 | `width_ge32=100` |

## Comparison

Packet 006 full TQ final15 at 100k:

- Recall: `0.9730` at nprobe32, `1.0000` at nprobe64.
- Latency: `5.14 / 5.72 / 6.11 ms` at nprobe32, `9.30 / 9.55 / 9.71 ms` at nprobe64.
- Storage: `100.8 MiB`, versus `22.5 MiB` for f32/source.
- TQ counters: full NEON, `scalar_candidates=0`, `width_ge32=100`.

Packet 009 width 75:

- Recall matches packet 006 at 100k: `0.9730` at nprobe32, `1.0000` at nprobe64.
- Latency improves nprobe32 tail to `4.86 / 5.46 / 5.63 ms`; nprobe64 remains effectively neutral at `9.07 / 9.41 / 9.72 ms`.
- Storage is worse than packet 006 full-width storage at `105.9 MiB`.

## Decision

Do not promote frontier-width reduction as the Task 124 solution.

The scorer is full SIMD, so the remaining blocker is not SIMD dispatch. The next TQ-specific implementation should target storage/locality/materialization: decoupled persisted TQ grouping, lower per-group sidecar overhead, or fused TQ score/top-k/materialization.
