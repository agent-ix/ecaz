---
task: 124
packet: 004
topic: tq-rerank-group-width
role: coder
date: 2026-06-29
head_sha: d542e0faec2f2799f12b5ee0ee1fa2f1fc9302aa
---

# Task 124 Review Request: TQ Rerank Group Width

This is a TurboQuant-focused iteration packet for Task 124. It is not a task closeout.

## Change

Code commit `d542e0faec2f2799f12b5ee0ee1fa2f1fc9302aa` adds `ec_ivf.rerank_group_width`, a build-time reloption for compact index-side rerank sidecars.

- `rerank_group_width=0` preserves prior behavior: groups flush at `rerank_width`.
- Nonzero values let us build smaller sidecar payload groups while keeping scan `rerank_width=100`.
- Validation limits the knob to `coarse_rerank` + index placement + compact rerank formats.
- The CLI `ec_ivf` profile recognizes `rerank_group_width` and `stage2_final_rerank_width` for clean suite runs.

## Evidence

Artifacts are under `reviews/task-124/004-tq-rerank-group-width/artifacts/`.

- `manifest.md`
- `task124-tq-group-width-100k-suite.json`
- `group-width-100k-manifest.json`
- `group-width-100k-results.jsonl`
- `group-width-100k-run.log`
- per-step load, recall, latency, storage, and explain logs

The suite is a 100k decision sweep across:

- f32/source baseline
- TQ default group width 100
- TQ group width 32
- TQ group width 16

All TQ variants use `rerank_width=100` and `stage2_final_rerank_width=25`.

## Result

Recall matches the f32 baseline at both nprobe settings:

| variant | nprobe32 recall@10 | nprobe64 recall@10 |
| --- | ---: | ---: |
| f32 source baseline | 0.9730 | 1.0000 |
| TQ group 100 | 0.9730 | 1.0000 |
| TQ group 32 | 0.9730 | 1.0000 |
| TQ group 16 | 0.9730 | 1.0000 |

Latency is mixed:

| variant | p50 nprobe32 | p95 nprobe32 | p99 nprobe32 | p50 nprobe64 | p95 nprobe64 | p99 nprobe64 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| f32 source baseline | 5.04 ms | 5.70 ms | 5.79 ms | 9.23 ms | 9.51 ms | 9.73 ms |
| TQ group 100 | 5.17 ms | 5.72 ms | 6.09 ms | 9.30 ms | 9.49 ms | 9.82 ms |
| TQ group 32 | 5.09 ms | 5.74 ms | 6.40 ms | 9.22 ms | 9.53 ms | 9.80 ms |
| TQ group 16 | 5.03 ms | 5.53 ms | 5.85 ms | 9.15 ms | 9.44 ms | 10.3 ms |

Storage gets worse for smaller groups:

| variant | index size |
| --- | ---: |
| f32 source baseline | 22.5 MiB |
| TQ group 100 | 100.8 MiB |
| TQ group 32 | 120.1 MiB |
| TQ group 16 | 120.2 MiB |

TQ stage2 remains fully SIMD:

- `scalar_candidates=0`
- `width_ge32=100`
- 10,000 TQ candidates scored per latency sweep point

The useful signal is sidecar locality. At 100k / nprobe64:

| variant | segment pages | segment bytes | header pages | header bytes | explain execution |
| --- | ---: | ---: | ---: | ---: | ---: |
| TQ group 100 | 216 | 1,748,632 | 45 | 274,449 | 13.926 ms |
| TQ group 32 | 87 | 653,382 | 67 | 500,916 | 13.347 ms |
| TQ group 16 | 33 | 146,982 | 75 | 577,346 | 12.723 ms |

## Request

Please review the reloption implementation and the interpretation.

My read: this is a useful diagnostic/tuning surface, but not a promotable TQ competitiveness win by itself. It proves the next work should attack sidecar/header storage overhead and/or coarse-scan dominance rather than the TQ scorer, which is already running SIMD in these runs.

Task 124 remains open. This packet does not satisfy the required 10k / 50k / 100k closeout matrix.
