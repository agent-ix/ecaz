# Task 73 Review Request: M5 SPIRE Recall/Latency Quality Gate

## Summary

This packet runs the M5-local SPIRE recall/latency Pareto sweep requested by Task 73 and includes an apples-to-apples IVF recall@10 control. The local recall ceiling is not capped around 0.90-0.93: SPIRE reaches recall@10 `1.0000` on 100k with permissive routing, so the scale drop is a tuning/defaults problem rather than evidence of a hard routing ceiling.

The useful AWS candidate from this packet is `top_graph_search_list_size=128`, `boundary_replica_count=0`, `nprobe=96` or `128`. The first gives recall@10 `0.9975` at p50 `75.790 ms` / p95 `79.387 ms` / p99 `82.456 ms`; the second gives recall@10 `1.0000` at p50 `95.960 ms` / p95 `96.476 ms` / p99 `99.049 ms`.

## Key Results

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE 10k default reproduction | tg16 b0 nprobe=16 | 0.9995 | 5.939 ms | 6.246 ms | 6.344 ms |
| SPIRE 100k current default shape | tg16 b0 nprobe=16 | 0.8525 | 13.505 ms | 15.410 ms | 15.868 ms |
| SPIRE 100k high-recall candidate | tg128 b0 nprobe=64 | 0.9825 | 51.227 ms | 54.958 ms | 59.428 ms |
| SPIRE 100k high-recall candidate | tg128 b0 nprobe=96 | 0.9975 | 75.790 ms | 79.387 ms | 82.456 ms |
| SPIRE 100k ceiling | tg128 b0 nprobe=128 | 1.0000 | 95.960 ms | 96.476 ms | 99.049 ms |
| IVF 100k control | nprobe=96, heap rerank 500 | 0.9980 | 10.6 ms | 11.9 ms | 14.0 ms |
| IVF 100k control | nprobe=128, heap rerank 500 | 1.0000 | 12.7 ms | 13.8 ms | 14.3 ms |

Boundary replicas improved lower-nprobe recall but made latency much worse on this host. `boundary_replica_count=1` reached recall@10 `0.9940` at nprobe 64 with p50 `108.444 ms`; `boundary_replica_count=2` reached recall@10 `0.9970` at nprobe 64 with p50 `167.272 ms`. Neither beats the b0 Pareto point for AWS.

## Interpretation

Task 73's load-bearing question is answered: permissive local settings reach 0.99+ and then 1.0000 recall@10, so SPIRE does not appear to have a hard 100k recall ceiling on this fixture. The visible issue is the default/fast operating point, where 100k recall@10 remains `0.8525` at nprobe 16.

This supports advancing to AWS for the selected high-recall points, but not as a blind speed profile of the current default. The AWS run should carry at least:

- current default: tg16 b0 nprobe=16
- high-recall candidate: tg128 b0 nprobe=96
- ceiling candidate: tg128 b0 nprobe=128
- IVF control at nprobe=96/128

## Artifacts

- Manifest: `reviews/task-73/001-spire-m5-quality-gate/artifacts/manifest.md`
- Suite config: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite.json`
- Suite manifest: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite-manifest.json`
- Structured results: `reviews/task-73/001-spire-m5-quality-gate/artifacts/results.jsonl`
- Raw logs: `reviews/task-73/001-spire-m5-quality-gate/artifacts/*.log`

## Validation

Ran `ecaz bench suite` on PG18 against local M5 fixture data. No code tests were run because this was a measurement packet with no code change under review.
