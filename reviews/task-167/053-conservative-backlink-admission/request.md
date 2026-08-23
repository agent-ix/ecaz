---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 conservative backlink admission

Status: review requested for candidate checkpoint `4826e9644`. No quality,
scale-matrix, merge, or closeout result is claimed by this code packet.

Packets 047 and 051 isolate two failed free-capacity backlink strategies at
50k. Robust-prune-all missed the fixed heldout band by `0.001611`; unconditional
append-when-room was `0.002000` worse. The former can remove existing edges
before capacity is exhausted, while the latter always injects a potentially
redundant backlink.

This candidate screens the union with the required exact-distance
`robust_prune`, but mutates a target with spare capacity only when the result
contains the new backlink and every existing edge. Otherwise it preserves the
target unchanged. Full targets retain ordinary re-pruning. This preserves
prior reachability and alpha-diversity while still admitting an exact-equivalent
inserted node without losing the old edge.

The production insert-work surface gains `backlink_prune_rejected`, and the
suite records it under the existing coordinator-backend scope. The rejected
unconditional-append control remains excluded until after candidate quality is
measured. Fixed packet-045 bands remain unchanged. All focused planner,
default, counter-reset, parser, and gate-control tests pass.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
