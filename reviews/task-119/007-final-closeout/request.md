---
task: 119
packet: reviews/task-119/007-final-closeout
checkpoint_sha: 0b911cf0dc2bd4be994bdd56e8fde248ac829b34
branch: task-119-hnsw-rabitq-coarse-rerank-profile
role: coder
date: 2026-06-25
---

# Review Request: Task 119 Final Closeout

## Summary

Task 119 is now complete as a benchmark/decision task.

The final recommendation is:

```text
keep experimental and iterate; do not promote HNSW RaBitQ coarse-rerank as a
production profile yet
```

The closeout evidence is in `artifacts/manifest.md`, with packet-local citations
to:

- Task 118 final attribution: `reviews/task-118/006-final-attribution-matrix/`
- Full Task 119 required matrix: `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/`
- Production-style read follow-up: `reviews/task-119/006-sidecar-rerank-m5-db-read/`

## What Is Complete

- Task 118 dependency is cited and interpreted: RaBitQ loss is dominated by
  candidate containment/traversal, not build-source-column A/B or a late rerank
  boundary.
- The Task 119 suite support now enumerates all required second-stage
  representations:
  `f32`, `rabitq2`, `rabitq4`, `rabitq8`, and `turboquant_2bit` through
  `turboquant_8bit`.
- Packet `005` measures the full required 10k/50k/100k matrix over the same
  RaBitQ-1 HNSW candidate frontier with explicit overfetch
  `ef_search={320,500,1000}` and `candidate_k=1000`.
- Packet `005` reports recall, latency, storage, frontier count, reranked count,
  source-read count, and emitted count for every required representation.
- Packet `006` adds production-style `tid-sorted` sidecar read evidence for the
  viable lanes: `f32`, `rabitq8`, `turboquant_4bit`, and `turboquant_8bit`.
- No durable storage layout change landed, so no version/lifecycle migration
  coverage is required in this task.

## Outcome

At 100k / `ef_search=1000` with production-style sidecar reads:

| Variant | Recall@10 | heap/source reads p50 | sidecar I/O p50 | score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `f32` | 0.9850 | 1000 | 23.285 ms | 37.768 ms | 86.750 ms | 6144 | 585.94 MiB |
| `rabitq8` | 0.9420 | 1000 | 6.611 ms | 9.266 ms | 41.170 ms | 1548 | 147.63 MiB |
| `turboquant_4bit` | 0.9415 | 1000 | 4.598 ms | 10.004 ms | 39.873 ms | 772 | 73.62 MiB |
| `turboquant_8bit` | 0.9760 | 1000 | 8.245 ms | 84.082 ms | 117.642 ms | 1540 | 146.87 MiB |

Interpretation:

- `f32` proves the overfetch/rerank recall ceiling, but it is too large and too
  slow to meet the storage-saving product goal.
- `turboquant_8bit` gets near the f32 recall ceiling but is score-latency
  dominated.
- `rabitq8` and `turboquant_4bit` are the practical compact lanes, but their
  100k recall is still roughly `0.942`, too far below f32 to promote.
- `turboquant_4bit` is the best compact Pareto lane in this harness because it
  is near `rabitq8` latency while using about half the sidecar bytes/vector.

## Remaining Work

No Task 119 acceptance blocker remains.

The remaining work is follow-up product work only:

- decide whether to open a new implementation task for an operator-visible
  experimental sidecar profile;
- optimize TurboQuant scoring if the `turboquant_8bit` recall point is worth
  pursuing;
- revisit durable HNSW storage layout only after a winning rerank
  representation is chosen;
- defer 1M until smaller-scale evidence shows a credible promotion candidate.

## Review Ask

Please review the closeout packet for evidence completeness and whether the
recommendation should be recorded as:

```text
keep experimental / iterate, do not promote
```
