---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 pruned-backlink no-op 50k gate

Status: preregistered; no runtime result, Task 167 acceptance, or closeout is
claimed.

Packet 056 introduces a full-target-only insertion candidate. When exact
`robust_prune` excludes a proposed backlink, the target retains its exact
established adjacency and order instead of being rewritten without the new
reverse edge. Spare-capacity robust-prune behavior and incomplete-population
stale-neighbor cleanup are unchanged.

This packet freezes one clean isolated 50k branch-point run before installing
or measuring the exact runtime. It retains packet 054's operating point: the
same 160 insert sources, 200 heldout queries, 48 inserted-neighborhood queries,
exact fp32 truth, graph/search settings, and packet-045 hard bands. The
append-when-room control cannot mutate the fixture before the candidate quality
gate passes.

The thresholds will not be widened after observing the run. Failure rejects
this candidate and leaves Task 167 open. A pass permits, but does not replace,
the required isolated 10k/50k/100k recall, latency, and storage matrix.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
