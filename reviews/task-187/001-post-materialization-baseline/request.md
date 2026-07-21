---
task: 187
packet: 001-post-materialization-baseline
role: coder
status: review_requested
date: 2026-07-21
---

# Review request: post-Task-191 traversal baseline

This packet freezes the fresh post-Task-191 `lazy10` baseline before selecting
any traversal candidate. It uses the retained `training_landmarks_exact`
policy, cap 4,096, 32 seeds, BW4/H100, graph degree 32, and RaBitQ neighbor
scoring on a byte-identical staged 100k generation and held-out query set.

The attribution run reset counters after warmups and captured recall,
result identity, warm latency distribution, storage/build identity, topology,
remote engagement, and traversal stage/work counters. No candidate is selected
from this packet; the measured decomposition gates packet 002.

## Runner contract

The checked-in `ecaz bench suite` config runs one three-owner physical fixture,
200 evaluation queries, 50 warm timed samples after 10 warmups, and stage
counters enabled only on the physical latency arm. Corpus/query data remain
staged out-of-band; packet artifacts contain only normalized results and compact
logs.

Result: the physical arm passed topology/serving/engagement. Recall was
0.9625 (95% Wilson interval 0.9532–0.9700); warm mean latency was 22.40 ms
(p50 22.20, p95 25.60, p99 26.80, max 27.30). Traversal accounted for
7.468 ms, with remote expansion 6.174 ms, local expansion 1.230 ms, and the
derived coordinator/frontier remainder 0.065 ms. Head scoring was 2.145 ms
and seed selection 0.094 ms. The dominant traversal cost is therefore the
remote owner request/response path, not frontier bookkeeping or head scoring.
Packet 002 uses this decomposition to decide whether any bounded candidate
is worth isolating.

Evidence and exact commands are indexed by
[`artifacts/manifest.md`](artifacts/manifest.md).
