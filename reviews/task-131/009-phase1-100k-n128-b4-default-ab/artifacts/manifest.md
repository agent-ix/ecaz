---
task: 131
packet: reviews/task-131/009-phase1-100k-n128-b4-default-ab
head_sha: d3a2743fe227a701625930e3cf50a54d9bb9d05f
date: 2026-07-01T06:42:36-07:00
---

# Task 131 Phase 1 100k n128/b4 Local Multi-Instance A/B

## Run

- Command: `target/debug/ecaz bench suite run --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json --only mi-100k-n128-b4-global-preheap-ab --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/100k-n128-b4-suite-manifest.json --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/100k-n128-b4-results.jsonl --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/100k-n128-b4-suite-run.log`
- Task bucket: `reviews/task-131`
- Packet path: `reviews/task-131/009-phase1-100k-n128-b4-default-ab`
- Lane: local multi-instance PG18
- Fixture: `ec_real_100k`
- Storage format: `rabitq`
- Index: SPIRE `n128/b4`
- Rerank mode: default
- Surface: isolated one-index-per-table local multi-instance surfaces
- Suite config source: `reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json`

## Artifacts

- `artifacts/100k-n128-b4/bench-suite/results.jsonl`
- `artifacts/100k-n128-b4/bench-suite/suite-manifest.json`
- `artifacts/100k-n128-b4/bench-suite/suite-run.log`
- `artifacts/100k-n128-b4/bench-suite/production-read-k10-baseline-default.log`
- `artifacts/100k-n128-b4/bench-suite/production-read-k10-global-preheap-on-default.log`
- `artifacts/100k-n128-b4/bench-suite/storage.log`
- `artifacts/100k-n128-b4-results.jsonl`
- `artifacts/100k-n128-b4-suite-manifest.json`
- `artifacts/100k-n128-b4-suite-run.log`

Generated split TSVs were pruned before packet copy; the packet contains no `*.tsv` or `*.tsv.gz`.

## Key Results

- Coordinator query metrics, baseline: recall `1.0000`, p50/p95/p99 `5366.063 / 6413.783 / 6711.519 ms`.
- Coordinator query metrics, global preheap on: recall `1.0000`, p50/p95/p99 `5357.062 / 6199.141 / 6616.327 ms`.
- Production-read total, baseline: p50/p95/p99 `5341 / 6259 / 6620 ms`.
- Production-read total, global preheap on: p50/p95/p99 `2668 / 3283 / 3784 ms`.
- Remote heap rows avoided: `6000 -> 2000`.
- Payload bytes: `0 -> 0` because this no-payload timeline run uses `--production-read-timeline-no-payload`.
- Global preheap pruned candidates: `4000` in both variants, with the opt-in gate changing which rows proceed to heap receive.
- Safety counters: `strict_fail_sum=0`, `timeout_sum=0`, `cancel_sum=0`, `degraded_skip_sum=0` for both variants.
- Storage: coordinator total `1.9 GiB`, indexes `394.5 MiB`, table `1.6 GiB`, rows `100000`.

## Load Timings

- Coordinator load completed in `951.82s`; coordinator index build `645.05s`.
- Remote node 2 load completed in `871.25s`; remote index build `594.97s`.
- Remote node 3 load completed in `862.69s`; remote index build `584.81s`.
- Remote node 4 load completed in `819.25s`; remote index build `559.34s`.
