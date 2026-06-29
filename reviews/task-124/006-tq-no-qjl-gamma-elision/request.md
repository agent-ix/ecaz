---
task: 124
packet: 006
topic: tq-no-qjl-gamma-elision
role: coder
date: 2026-06-29
head_sha: 2744aa4f2c0290ce2b44f20ae1177518e416fab5
---

# Task 124 Review Request: TQ No-QJL Gamma Elision

This is a TurboQuant-focused optimization slice. It is not a task closeout.

## What Changed

TurboQuant rerank sidecar payloads now omit the stored `f32` gamma when the
active TQ scorer does not use QJL exact scoring. For the measured 1536D 4-bit
lane, this changes the sidecar payload floor from `772` bytes to `768` bytes.
QJL-active TQ dimensions still store gamma.

The code path under review is `src/am/ec_ivf/rerank.rs`:

- resolves whether the dimension's TQ scorer needs gamma
- encodes no-QJL payloads as code-only bytes
- keeps QJL-active payloads as gamma plus code
- decodes no-QJL payloads with a dummy gamma that is ignored by the no-QJL scorer

## Evidence

Artifacts are under `reviews/task-124/006-tq-no-qjl-gamma-elision/artifacts/`.

- `manifest.md`
- `task124-tq-no-qjl-gamma-elision-final15-ab-suite.json`
- `suite-run-r2.log`
- `final15-ab-suite-r2/suite-manifest.json`
- `final15-ab-suite-r2/results.jsonl`
- per-step load, recall, latency, and storage logs under `final15-ab-suite-r2/`

Truth caches are intentionally untracked.

## Key Outcome

The change is safe, but it does not materially improve TQ competitiveness.

Recall:

| scale | f32 nprobe32 | TQ final15 nprobe32 | f32 nprobe64 | TQ final15 nprobe64 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 50k | 0.9960 | 0.9960 | 1.0000 | 0.9990 |
| 100k | 0.9730 | 0.9730 | 1.0000 | 1.0000 |

Latency:

| scale | variant | nprobe32 p50/p95/p99 | nprobe64 p50/p95/p99 |
| --- | --- | --- | --- |
| 10k | f32 | 0.80 / 0.91 / 1.02 ms | 1.29 / 1.41 / 1.49 ms |
| 10k | TQ final15 | 0.76 / 0.86 / 1.08 ms | 1.23 / 1.43 / 1.71 ms |
| 50k | f32 | 2.47 / 2.67 / 2.82 ms | 4.87 / 5.04 / 5.15 ms |
| 50k | TQ final15 | 2.43 / 2.68 / 2.89 ms | 4.83 / 4.98 / 5.09 ms |
| 100k | f32 | 5.13 / 5.57 / 5.83 ms | 9.23 / 9.39 / 9.55 ms |
| 100k | TQ final15 | 5.14 / 5.72 / 6.11 ms | 9.30 / 9.55 / 9.71 ms |

Storage:

| scale | f32/source index | TQ final15 index |
| --- | ---: | ---: |
| 10k | 2.9 MiB | 10.9 MiB |
| 50k | 11.6 MiB | 50.8 MiB |
| 100k | 22.5 MiB | 100.8 MiB |

TQ scorer counters remain fully SIMD at every measured point:
`isa=neon`, `scalar_candidates=0`, `width_ge32=100`.

## Request

Please review the code and interpretation.

My read: this cleanup is worth keeping because it removes unnecessary persisted
bytes and keeps QJL safety intact, but it proves that the current blocker is not
gamma. Task 124 still needs a deeper TQ-specific payload/layout optimization.
