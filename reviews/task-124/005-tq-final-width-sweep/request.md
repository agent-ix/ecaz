---
task: 124
packet: 005
topic: tq-final-width-sweep
role: coder
date: 2026-06-29
head_sha: e5c1bf254360f45ec088bea82550f9a58238a901
---

# Task 124 Review Request: TQ Final Width Sweep

This is a TurboQuant-focused benchmark packet. It is not a task closeout.

## What This Tests

Packet 004 showed the TQ scorer is already full SIMD and that group-width locality helps sidecar reads but does not fix storage or end-to-end latency enough. This packet tests the next low-risk lever: reducing the TQ stage2 final exact f32 pass width.

The 100k decision sweep tested final widths 10/15/20/25. Final10 broke recall, while final15 and final20 preserved the same 100k recall as final25. I then ran a full 10k / 50k / 100k A/B for:

- f32/source baseline, `rerank_width=100`
- TQ index-side sidecar, `rerank_width=100`, `stage2_final_rerank_width=15`

## Evidence

Artifacts are under `reviews/task-124/005-tq-final-width-sweep/artifacts/`.

- `manifest.md`
- `task124-tq-final-width-100k-suite.json`
- `final-width-100k-manifest.json`
- `final-width-100k-results.jsonl`
- `task124-tq-final15-ab-10-50-100-suite.json`
- `final15-ab-manifest.json`
- `final15-ab-results.jsonl`
- per-step load, recall, latency, storage, and explain logs

Truth caches are intentionally untracked.

## Key Outcome

Final15 is better than final25 as a candidate configuration, but it is not enough to close Task 124.

Recall:

| scale | f32 nprobe32 | TQ final15 nprobe32 | f32 nprobe64 | TQ final15 nprobe64 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 50k | 0.9960 | 0.9960 | 1.0000 | 0.9990 |
| 100k | 0.9730 | 0.9730 | 1.0000 | 1.0000 |

Latency:

| scale | variant | nprobe32 p50/p95/p99 | nprobe64 p50/p95/p99 |
| --- | --- | --- | --- |
| 10k | f32 | 0.78 / 0.91 / 1.02 ms | 1.29 / 1.43 / 1.55 ms |
| 10k | TQ final15 | 0.74 / 0.86 / 1.10 ms | 1.17 / 1.33 / 1.46 ms |
| 50k | f32 | 2.89 / 4.02 / 4.54 ms | 5.40 / 6.68 / 7.36 ms |
| 50k | TQ final15 | 2.42 / 2.68 / 2.89 ms | 4.90 / 5.16 / 5.34 ms |
| 100k | f32 | 5.42 / 6.03 / 6.21 ms | 9.46 / 10.6 / 12.6 ms |
| 100k | TQ final15 | 5.23 / 5.73 / 6.01 ms | 9.87 / 12.9 / 15.0 ms |

Storage:

| scale | f32/source index | TQ final15 index |
| --- | ---: | ---: |
| 10k | 2.9 MiB | 10.9 MiB |
| 50k | 11.6 MiB | 50.8 MiB |
| 100k | 22.5 MiB | 100.8 MiB |

TQ scorer counters remain fully SIMD: `scalar_candidates=0`, `width_ge32=100`, 10,000 TQ candidates per latency sweep point.

## Request

Please review the interpretation.

My read: final15 is the best TQ final-width setting found so far, but Task 124 remains open because storage is still far too high, 50k/nprobe64 recall is not exactly matched, and 100k/nprobe64 tail latency regressed in this run. The next optimization should attack sidecar storage/header overhead and 100k tail behavior.
