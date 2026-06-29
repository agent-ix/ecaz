# Task 124 Packet 010: TQ Group-Width Decision

This is a TurboQuant-focused diagnostic benchmark packet. It is not Task 124 closeout.

## Summary

Packet 009 showed that runtime `rerank_width=75` preserves 100k full-TQ4 recall, but the default persisted group width of 75 increased storage to `105.9 MiB`. The code already exposes a build-time `rerank_group_width` reloption, so this packet tests whether keeping scan-time width 75 while decoupling persisted sidecar grouping can recover storage without hurting recall or tail latency.

The tested pipeline remains Task 124's target shape:

- coarse RaBitQ frontier
- index-side full 4-bit TurboQuant stage-2 rerank with runtime `rerank_width=75`
- final exact f32 rerank at `stage2_final_rerank_width=15`

## Code Commit

No code change in this packet.

Code under test:

- `381fcdc81323168961b4593232aba2056763cffc`

## Benchmark Evidence

Suite config:

- `reviews/task-124/010-tq-group-width-decision/artifacts/task124-tq-group-width-100k-suite.json`

Suite command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/010-tq-group-width-decision/artifacts/task124-tq-group-width-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/010-tq-group-width-decision/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/010-tq-group-width-decision/artifacts/group-width-100k/suite-manifest.json
```

Report summary: `completed 8`, `failed 0`, `skipped 0`.

Artifacts:

- `reviews/task-124/010-tq-group-width-decision/artifacts/manifest.md`
- `reviews/task-124/010-tq-group-width-decision/artifacts/suite-run.log`
- `reviews/task-124/010-tq-group-width-decision/artifacts/group-width-100k/suite-manifest.json`
- `reviews/task-124/010-tq-group-width-decision/artifacts/group-width-100k/results.jsonl`
- per-step load, recall, latency, and storage logs under `group-width-100k/`

Truth cache files are intentionally untracked.

## Results

Recall at k=10:

| Runtime width | Group width | nprobe=32 | nprobe=64 |
| ---: | ---: | ---: | ---: |
| 75 | 50 | 0.9730 | 1.0000 |
| 75 | 100 | 0.9730 | 1.0000 |

Latency:

| Runtime width | Group width | nprobe | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 75 | 50 | 32 | 4.90 ms | 5.47 ms | 5.70 ms |
| 75 | 50 | 64 | 9.12 ms | 9.48 ms | 9.79 ms |
| 75 | 100 | 32 | 5.36 ms | 7.04 ms | 8.12 ms |
| 75 | 100 | 64 | 9.70 ms | 10.4 ms | 10.8 ms |

Storage:

| Runtime width | Group width | ec_ivf index size | per row |
| ---: | ---: | ---: | ---: |
| 75 | 50 | 100.8 MiB | 1057.2 B |
| 75 | 100 | 100.8 MiB | 1056.6 B |

TQ scorer counters remain full SIMD:

| Runtime width | Group width | nprobe | quant | isa | scalar_candidates | TQ candidates |
| ---: | ---: | ---: | --- | --- | ---: | ---: |
| 75 | 50 | 32 | turboquant | neon | 0 | 7500 |
| 75 | 50 | 64 | turboquant | neon | 0 | 7500 |
| 75 | 100 | 32 | turboquant | neon | 0 | 7500 |
| 75 | 100 | 64 | turboquant | neon | 0 | 7500 |

## Interpretation

`rerank_group_width=50` is the better group-width setting among these two variants. It preserves width-75 recall and recovers storage from packet 009's `105.9 MiB` width-75 default to `100.8 MiB`, matching the packet 006 full-width TQ storage. It does not materially improve latency versus packet 009 width 75, but it avoids the group-100 tail regression.

`rerank_group_width=100` is not useful here. It preserves recall and storage, but worsens 100k latency substantially: nprobe32 p99 rises to `8.12 ms` and nprobe64 p99 to `10.8 ms`.

The scan code explains the shape: the direct group loader copies only selected payloads, but it still follows payload segment chains far enough to reach those selected payloads. Larger persisted groups reduce some layout overhead but can make selected-payload access less local for TQ's large 768-byte payloads.

## Decision

Do not promote group-width tuning as the Task 124 solution.

The best current full-TQ4 configuration from these decision sweeps is `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`, but it is still not competitive with the f32/source baseline because 100k TQ index storage remains `100.8 MiB` versus `22.5 MiB`.

Task 124 remains open. The next implementation work should target the layout format itself: payload-segment locality/direct addressing, less duplicated TQ sidecar bytes, or a fused stage-2 path that avoids touching as many 768-byte payloads.
