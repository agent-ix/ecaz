# Task 124 Packet 010 Artifact Manifest

- Task bucket: `reviews/task-124/`
- Packet path: `reviews/task-124/010-tq-group-width-decision/`
- Head SHA: `381fcdc81323168961b4593232aba2056763cffc`
- Timestamp: `2026-06-29T04:10:00Z`
- Lane: local PG18, release extension install from prior Task 124 work, `ec_ivf` staged real 100k corpus
- Storage format: `coarse_rerank`
- Coarse frontier: `coarse_format=rabitq`, `coarse_bits=1`, `nlists=64`, recall/latency sweeps at `nprobe=32,64`
- Stage-2 variant: `rerank_placement=index`, `rerank_format=turboquant`, runtime `rerank_width=75`, `stage2_final_rerank_width=15`
- Group-width sweep: `rerank_group_width=50,100`
- Fixture isolation: one index/table prefix per group width: `task124_tq_w75_g50_100k`, `task124_tq_w75_g100_100k`
- Outcome: diagnostic only; group width 50 is better than 100 and recovers the width-75 storage penalty, but full-TQ4 storage remains far above f32/source.

## Code Under Test

No code change was introduced for this packet.

Commit under test:

- `381fcdc81323168961b4593232aba2056763cffc`

## Suite Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/010-tq-group-width-decision/artifacts/task124-tq-group-width-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/010-tq-group-width-decision/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/010-tq-group-width-decision/artifacts/group-width-100k/suite-manifest.json
```

Report summary: `completed 8`, `failed 0`, `skipped 0`.

## Committed Artifacts

- `task124-tq-group-width-100k-suite.json`: SuiteConfig for the 100k group-width decision sweep.
- `suite-run.log`: suite runner log.
- `group-width-100k/suite-manifest.json`: structured suite manifest.
- `group-width-100k/results.jsonl`: structured result records.
- `group-width-100k/load-*.log`: per-variant load logs.
- `group-width-100k/recall-*.log`: per-variant recall logs.
- `group-width-100k/latency-*.log`: per-variant latency logs with candidate-batch counters.
- `group-width-100k/storage-*.log`: per-variant storage logs.

Regenerable `truth-*.json` caches were intentionally not committed.

## Key Results

### Recall at k=10

| Runtime width | Group width | nprobe=32 | nprobe=64 |
| ---: | ---: | ---: | ---: |
| 75 | 50 | 0.9730 | 1.0000 |
| 75 | 100 | 0.9730 | 1.0000 |

### Latency

| Runtime width | Group width | nprobe | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 75 | 50 | 32 | 4.90 ms | 5.47 ms | 5.70 ms |
| 75 | 50 | 64 | 9.12 ms | 9.48 ms | 9.79 ms |
| 75 | 100 | 32 | 5.36 ms | 7.04 ms | 8.12 ms |
| 75 | 100 | 64 | 9.70 ms | 10.4 ms | 10.8 ms |

### Storage

| Runtime width | Group width | ec_ivf index size | per row |
| ---: | ---: | ---: | ---: |
| 75 | 50 | 100.8 MiB | 1057.2 B |
| 75 | 100 | 100.8 MiB | 1056.6 B |

### Kernel Counters

| Runtime width | Group width | nprobe | quant | isa | scalar_candidates | TQ candidates |
| ---: | ---: | ---: | --- | --- | ---: | ---: |
| 75 | 50 | 32 | turboquant | neon | 0 | 7500 |
| 75 | 50 | 64 | turboquant | neon | 0 | 7500 |
| 75 | 100 | 32 | turboquant | neon | 0 | 7500 |
| 75 | 100 | 64 | turboquant | neon | 0 | 7500 |

## Comparison

Packet 009 width 75 with default group width 75:

- Recall: `0.9730` at nprobe32, `1.0000` at nprobe64.
- Latency: `4.86 / 5.46 / 5.63 ms` at nprobe32, `9.07 / 9.41 / 9.72 ms` at nprobe64.
- Storage: `105.9 MiB`, `1110.0 B/row`.

Packet 010 width 75 with group width 50:

- Recall: `0.9730` at nprobe32, `1.0000` at nprobe64.
- Latency: `4.90 / 5.47 / 5.70 ms` at nprobe32, `9.12 / 9.48 / 9.79 ms` at nprobe64.
- Storage: `100.8 MiB`, `1057.2 B/row`.

Packet 010 width 75 with group width 100:

- Recall: `0.9730` at nprobe32, `1.0000` at nprobe64.
- Latency: `5.36 / 7.04 / 8.12 ms` at nprobe32, `9.70 / 10.4 / 10.8 ms` at nprobe64.
- Storage: `100.8 MiB`, `1056.6 B/row`.

## Decision

Do not promote group-width tuning as the Task 124 solution.

Group width 50 is a better companion for runtime width 75 than group width 75 or 100, but the improvement is incremental and does not resolve the storage gap. The next TQ-specific implementation should reduce the TQ sidecar bytes touched or persisted, not just tune group boundaries.
