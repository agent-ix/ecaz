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

The attribution run must reset counters after warmups and capture recall,
result identity, warm latency distribution, storage/build identity, topology,
remote engagement, and traversal stage/work counters. No candidate is selected
from this packet; the measured decomposition gates packet 002.

## Runner contract

The checked-in `ecaz bench suite` config runs one three-owner physical fixture,
200 evaluation queries, 50 warm timed samples after 10 warmups, and stage
counters enabled only on the physical latency arm. Corpus/query data remain
staged out-of-band; packet artifacts contain only normalized results and compact
logs.

Evidence and exact commands are indexed by
[`artifacts/manifest.md`](artifacts/manifest.md).
