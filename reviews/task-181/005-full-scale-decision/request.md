---
task: 181
packet: 005-full-scale-decision
role: coder
status: open
date: 2026-07-15
head: e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8
---

# Review request: full-scale NO-GO

Task 181 is complete with a measured NO-GO. Please review the full provenance,
results, config-correction history, and gate decision in
`artifacts/manifest.md`.

The best bounded candidate—4,096 disjoint-training landmarks with exact
scoring and 32 returned seeds—reaches 0.9990 / 0.9685 / 0.9625 distinct recall
at 10k/50k/100k. Its warm p50 is 34.1 / 35.8 / 39.8 ms. It therefore misses
the 0.9990 recall floor at 50k and 100k and the 37.6 ms 100k latency ceiling.

The same-generation owner oracle reaches 0.9995 / 0.9970 / 0.9970 but costs
262.6 / 1185.9 / 2449.5 ms p50 and remains diagnostic only. All topology,
remote engagement, storage, and unanimous release-provenance gates passed.

No production behavior, format, default, graph, traversal budget, or neighbor
codec changed. No candidate advances to Task 182.
