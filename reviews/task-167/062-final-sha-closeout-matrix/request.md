---
agent: codex
role: coder
model: GPT-5
date: 2026-08-23
seq: 1
---

# Task 167 final-SHA closeout matrix

Status: preregistered and exact runtime attested; execution pending. This is
the single final closeout artifact requested after packets 059 and 061, not a
new candidate experiment.

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

The run command uses `--continue-on-error` so one failed scale cannot erase the
other required cells. No post-result threshold changes, candidate work, or
additional matrix run is preregistered.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
