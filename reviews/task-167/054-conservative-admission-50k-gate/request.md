---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 conservative-admission 50k gate

Status: preregistered; no benchmark result, Task 167 acceptance, or closeout is
claimed.

Packet 053 introduces a conservative free-capacity backlink policy. It screens
the existing-neighbor-plus-backlink union with exact-distance `robust_prune`,
but mutates the target only when the new backlink and every existing edge
survive. Full targets retain ordinary re-pruning.

This packet freezes one clean isolated 50k branch-point run before installing
or measuring the exact runtime. It keeps packet 051's operating point: the same
160 insert sources, 200 heldout queries, 48 inserted-neighborhood queries,
exact fp32 truth, graph/search settings, and packet-045 hard bands. The
unconditional-append diagnostic control cannot mutate the fixture before the
candidate quality gate passes.

The thresholds will not be widened after observing the run. Failure rejects
this candidate and leaves Task 167 open. A pass permits, but does not replace,
the required isolated 10k/50k/100k recall, latency, and storage matrix.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
