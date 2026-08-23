---
agent: codex
role: coder
model: GPT-5
date: 2026-08-23
seq: 1
---

# Task 167 final-SHA closeout matrix

Status: review-open. The single preregistered final-SHA closeout run completed
all three cells successfully. This is the closeout artifact requested after
packets 059 and 061, not a new candidate experiment.

The immutable suite config runs the final shipped insertion path once at each
required real-corpus scale: 10k, 50k, and 100k. Every cell uses PG18, three
physical owners, graph degree 32, head cap 4,096, beam width 4, candidate heap
32, hop cap 100, 200 ordinary/heldout queries, and 48 additional
inserted-neighborhood queries. It records physical and single-control recall,
latency, storage, insert throughput/work, and post-insert exact recall.

The inserted-neighborhood AC-4 gate remains hard. Heldout rows run in packet
061's non-blocking baseline-recording mode and are disclosed per scale; they do
not carry an absolute quality verdict. Fault and concurrency drills are skipped
because this packet closes the outstanding scale matrix and those Task 167
correctness surfaces already have packet evidence.

The run used `--continue-on-error` so one failed scale could not erase the
other required cells. No post-result threshold changes, candidate work, or
additional matrix run occurred.

## Result

All three isolated cells succeeded on the exact release build at
`3da8c572ec5a1034ef5563c661da201c8ad83efe`.

| Scale | Ordinary physical recall | Latency mean / p95 | Graph-side bytes / raw-vector amplification | Inserted-neighborhood physical / fresh / deficit | AC-4 | Heldout physical / fresh / physical-minus-fresh |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| 10k | 0.9990 | 15.20 / 17.50 ms | 76,095,488 / 1.238533x | 0.931920 / 0.935681 / 0.003762 | PASS | 0.973000 / 0.974500 / -0.001500 |
| 50k | 0.9545 | 20.70 / 22.50 ms | 410,214,400 / 1.335333x | 0.916791 / 0.931052 / 0.014261 | PASS | 0.843722 / 0.857333 / -0.013611 |
| 100k | 0.9295 | 17.00 / 18.30 ms | 831,782,912 / 1.353813x | 0.916419 / 0.922082 / 0.005663 | PASS | 0.805500 / 0.767000 / +0.038500 |

The inserted-neighborhood deficit stayed within the preregistered 0.015 AC-4
band at every scale. Heldout is disclosed in the requested baseline-recording
mode and carries no absolute quality verdict. The 100k cell supplies the first
clean post-isolation 100k AC-4 measurement, and all recall, latency, and storage
rows trace to this final build.

Review request: accept packet 062 as Task 167's final measurement evidence and
close Task 167. No further candidate or benchmark run is requested.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
