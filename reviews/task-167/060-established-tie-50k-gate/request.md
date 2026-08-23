---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 established-tie 50k gate

Status: superseded unrun by packet 059 reviewer feedback. The reviewer accepted
the established-tie code as a correctness alignment and rejected this packet's
cross-scale absolute heldout gate semantics. No runtime result exists, and this
configuration must not run.

Packet 059 aligns the physical backlink robust-prune union with the
established-first order used by the pure/local planners. Exact-distance ties
are therefore resolved in favor of an established neighbor rather than a new
exact-vector duplicate that happened to occupy temporary union ordinal zero.

This packet freezes one clean isolated 50k branch-point run before installing
or measuring the exact runtime. It retains packet 057's operating point: the
same 160 exact-duplicate insert sources, 200 heldout queries, 48
inserted-neighborhood queries, exact fp32 truth, graph/search settings, and
packet-045 hard bands. The append-when-room control cannot mutate the fixture
before the candidate quality gate passes.

Packet 059 feedback at
`reviews/task-167/059-established-tie-priority/feedback/2026-08-23-01-reviewer.md`
directs the coder to land the correctness
alignment without a quality verdict, replace the heldout gate with a per-scale
baseline-relative regression detector, and run the final isolated
10k/50k/100k matrix. Packet 061 implements that gate correction. This packet's
preregistered config is retained only as historical provenance.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
