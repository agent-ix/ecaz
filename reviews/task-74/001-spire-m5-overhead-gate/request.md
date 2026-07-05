# Task 74 Review Request: M5 SPIRE Overhead Gate

## Summary

This packet evaluates Task 74 using the Task 73-selected local settings. The M5 evidence says AWS profiling is worthwhile, but the AWS matrix should focus on the high-recall b0 path plus IVF controls, not boundary replicas and not only the current default.

The local overhead gap is large at comparable recall: SPIRE tg128 b0 nprobe=96 reaches recall@10 `0.9975` at p50 `75.790 ms`, while IVF nprobe=96 reaches recall@10 `0.9980` at p50 `10.6 ms`. At the ceiling, SPIRE tg128 b0 nprobe=128 reaches recall@10 `1.0000` at p50 `95.960 ms`, while IVF nprobe=128 reaches recall@10 `1.0000` at p50 `12.7 ms`.

## Findings

The current default is fast but low recall: tg16 b0 nprobe=16 gives recall@10 `0.8525` at p50 `13.505 ms`. Optimizing only that path would optimize the wrong shape.

The best AWS SPIRE candidates are:

- tg128 b0 nprobe=96: recall@10 `0.9975`, p50 `75.790 ms`, p95 `79.387 ms`, p99 `82.456 ms`
- tg128 b0 nprobe=128: recall@10 `1.0000`, p50 `95.960 ms`, p95 `96.476 ms`, p99 `99.049 ms`

Boundary replicas improve recall at lower nprobe but are slower on M5. b1 nprobe=64 gives recall@10 `0.9940` at p50 `108.444 ms`; b2 nprobe=64 gives recall@10 `0.9970` at p50 `167.272 ms`.

## Artifacts

- Packet-local summary: `reviews/task-74/001-spire-m5-overhead-gate/artifacts/overhead-summary.md`
- Manifest: `reviews/task-74/001-spire-m5-overhead-gate/artifacts/manifest.md`
- Source suite and raw logs: `reviews/task-73/001-spire-m5-quality-gate/artifacts/`

## Validation

Ran `ecaz bench suite` on PG18 against local M5 fixture data. No separate external profiler was installed or run; this packet uses suite-visible query metrics, SPIRE pipeline counters, local production-read totals, and the IVF same-host control.
