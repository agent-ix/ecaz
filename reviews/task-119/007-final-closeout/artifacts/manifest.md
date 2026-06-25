---
head_sha: 0b911cf0dc2bd4be994bdd56e8fde248ac829b34
task: task-119
packet: reviews/task-119/007-final-closeout
host_class: m5-local
date: 2026-06-25
---

# Task 119 Final Closeout Manifest

## Scope

This closeout covers Task 119 as a measurement task:

```text
RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

The decision is based on M5-local `ecaz bench suite` evidence and the Task 118
attribution dependency. No durable HNSW storage layout change is promoted by
this packet.

## Dependency: Task 118

Task 118 is complete in `plan/tasks/118-hnsw-quantized-recall-attribution.md`
and cites `reviews/task-118/006-final-attribution-matrix/` as the final M5
release evidence packet.

Task 118 outcome relevant to Task 119:

- RaBitQ recall loss is dominated by candidate containment/traversal, not
  source-vs-compressed graph build.
- At 100k / `ef_search=200`, RaBitQ source-build recall is `0.8990`, while
  truth@10 in the emitted pool is `0.9325`.
- Source-build and compressed-build recall match for RaBitQ at every measured
  scale, so build-source-column A/B does not explain the loss.
- RaBitQ score correlation is comparatively strong, but its candidate
  containment is lower than the stronger HNSW lanes.

This unblocks Task 119 only as a measured overfetch/rerank viability test. It
does not justify promotion without Task 119 showing a credible recall, latency,
and storage Pareto point.

Task 118 artifacts cited:

| Artifact | Purpose |
| --- | --- |
| `reviews/task-118/006-final-attribution-matrix/request.md` | Final Task 118 M5 release interpretation |
| `reviews/task-118/006-final-attribution-matrix/artifacts/manifest.md` | Provenance, commands, suite status |
| `reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-m5-release.txt` | Compact 10k/50k/100k decision table |

## Task 119 Evidence Packets

| Packet | Status | Purpose |
| --- | --- | --- |
| `reviews/task-119/002-sidecar-rerank-matrix-support/` | complete | Adds measured sidecar variants for `f32`, `rabitq2/4/8`, and `turboquant_2bit` through `turboquant_8bit` |
| `reviews/task-119/004-sidecar-counter-columns/` | complete | Adds explicit frontier/reranked/read/emitted counter columns |
| `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/` | complete | Full required 10k/50k/100k matrix, 11 variants x `ef_search={320,500,1000}`, free-I/O upper bound |
| `reviews/task-119/006-sidecar-rerank-m5-db-read/` | complete | Production-style `tid-sorted` sidecar reads for the viable lanes |

Packet `003` is superseded by packet `005` because packet `005` reran the same
full matrix with explicit counters.

## Required Matrix Coverage

Packet `005` covers every required rerank representation at 10k, 50k, and 100k:

- `f32`
- `rabitq2`
- `rabitq4`
- `rabitq8`
- `turboquant_2bit`
- `turboquant_3bit`
- `turboquant_4bit`
- `turboquant_5bit`
- `turboquant_6bit`
- `turboquant_7bit`
- `turboquant_8bit`

Each scale has `33` result rows: `11` variants x `3` `ef_search` values. The
measured overfetch settings were `ef_search={320,500,1000}` and
`candidate_k=1000`.

The `1536`-dimensional `turboquant_4bit` lane in this matrix is the special
tiled no-QJL lane: 4 MSE bits/dim, 0 QJL bits/dim, and 16 MSE centroids. The
other TurboQuant lanes use the QJL-active Task 119 composition:

| TurboQuant label | MSE bits/dim | QJL bits/dim | MSE centroids |
| --- | ---: | ---: | ---: |
| `turboquant_2bit` | 1 | 1 | 2 |
| `turboquant_3bit` | 2 | 1 | 4 |
| `turboquant_4bit` | 3 | 1 | 8 |
| `turboquant_5bit` | 4 | 1 | 16 |
| `turboquant_6bit` | 5 | 1 | 32 |
| `turboquant_7bit` | 6 | 1 | 64 |
| `turboquant_8bit` | 7 | 1 | 128 |

## Key Free-I/O Results

Packet `005`, `ef_search=1000`:

| Scale | Variant | Recall@10 | frontier p50 | reranked p50 | heap/source reads p50 | emitted p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 1000 | 1000 | 0 | 10 | 39.315 ms | 6144 | 58.59 MiB |
| 10k | `rabitq8` | 0.9650 | 1000 | 1000 | 0 | 10 | 25.888 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 1000 | 1000 | 0 | 10 | 26.171 ms | 772 | 7.36 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 1000 | 1000 | 0 | 10 | 98.752 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 1000 | 1000 | 0 | 10 | 39.334 ms | 6144 | 292.97 MiB |
| 50k | `rabitq8` | 0.9475 | 1000 | 1000 | 0 | 10 | 26.326 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 1000 | 1000 | 0 | 10 | 26.863 ms | 772 | 36.81 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 1000 | 1000 | 0 | 10 | 102.636 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 1000 | 1000 | 0 | 10 | 47.411 ms | 6144 | 585.94 MiB |
| 100k | `rabitq8` | 0.9420 | 1000 | 1000 | 0 | 10 | 34.405 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 1000 | 1000 | 0 | 10 | 35.115 ms | 772 | 73.62 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 1000 | 1000 | 0 | 10 | 112.756 ms | 1540 | 146.87 MiB |

## Key DB-Read Results

Packet `006`, `ef_search=1000`, `read_mode=tid-sorted`:

| Scale | Variant | Recall@10 | frontier p50 | heap/source reads p50 | sidecar I/O p50 | score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 1000 | 1000 | 23.917 ms | 38.131 ms | 77.489 ms | 6144 | 58.59 MiB |
| 10k | `rabitq8` | 0.9650 | 1000 | 1000 | 5.658 ms | 9.424 ms | 30.689 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 1000 | 1000 | 3.472 ms | 9.991 ms | 29.107 ms | 772 | 7.36 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 1000 | 1000 | 6.144 ms | 82.553 ms | 104.535 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 1000 | 1000 | 23.158 ms | 38.488 ms | 80.183 ms | 6144 | 292.97 MiB |
| 50k | `rabitq8` | 0.9475 | 1000 | 1000 | 6.356 ms | 9.502 ms | 33.980 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 1000 | 1000 | 4.070 ms | 10.106 ms | 32.386 ms | 772 | 36.81 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 1000 | 1000 | 6.826 ms | 86.588 ms | 111.725 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 1000 | 1000 | 23.285 ms | 37.768 ms | 86.750 ms | 6144 | 585.94 MiB |
| 100k | `rabitq8` | 0.9420 | 1000 | 1000 | 6.611 ms | 9.266 ms | 41.170 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 1000 | 1000 | 4.598 ms | 10.004 ms | 39.873 ms | 772 | 73.62 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 1000 | 1000 | 8.245 ms | 84.082 ms | 117.642 ms | 1540 | 146.87 MiB |

## Acceptance Criteria Audit

| Criterion | Result |
| --- | --- |
| Cite Task 118 and make go/no-go explicit | Satisfied. Task 118 shows RaBitQ loss is candidate containment/traversal; Task 119 proceeds only as a measured overfetch/rerank viability test. |
| Measure true RaBitQ-1 coarse-rerank with required matrix | Satisfied by packet `005`: all required second-stage variants over a RaBitQ-1 HNSW candidate frontier. |
| Report recall, latency, storage, candidate-stage counters at 10k/50k/100k | Satisfied by packet `005` for all required variants. Packet `006` adds nonzero sidecar read counts for viable lanes. |
| Recommend promote / keep experimental / iterate / shelve | Satisfied. Recommendation: keep experimental and iterate; do not promote. |
| Durable storage layout changes include version/lifecycle coverage | Not applicable. No durable layout change landed. |

## Final Decision

Do not promote HNSW RaBitQ coarse-rerank as a production profile yet.

The measured profile is useful enough to keep experimental, with future
iteration focused on `turboquant_4bit` and `rabitq8` style compact rerank
lanes:

- `f32` is the recall ceiling but violates the storage-saving goal and is slow
  once production-style sidecar reads are included.
- `turboquant_8bit` preserves high recall but is dominated by score latency.
- `rabitq8` and `turboquant_4bit` are the only compact practical lanes, but at
  100k their recall remains around `0.942`, well below the f32 ceiling of
  `0.985`.
- `turboquant_4bit` is the best compact storage/latency lane in this harness,
  using about half the sidecar bytes/vector of `rabitq8` with nearly the same
  total-bound latency.

## Remaining Work Outside Task 119

These are follow-up directions, not blockers to closing Task 119:

- Open a new implementation task if the project wants to turn the sidecar
  harness into an operator-visible HNSW profile.
- Optimize the TurboQuant scorer if `turboquant_8bit` recall is worth pursuing.
- Revisit durable HNSW storage layout only after selecting a winning rerank
  representation.
- Add 1M evidence only if later 10k/50k/100k iterations show a credible
  promotion candidate.
