---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 pruned-backlink no-op

Status: review requested for candidate checkpoints `5e32a1dfb` and
`3da6df06c`. No quality, scale-matrix, merge, or closeout result is claimed by
this code packet.

Packets 047, 051, and 054 changed spare-capacity backlink policy, but every
candidate retained the same full-target behavior: after `robust_prune`, the
distributed path rewrote the target even when the proposed backlink was absent
from the result. The mature local `ec_diskann` incremental planner explicitly
returns no mutation in that case.

This candidate applies that rule to both DistANN planners. A full target whose
prune rejects the proposed backlink keeps its exact prior adjacency and order.
The physical guard is explicitly capacity-scoped, so it neither changes the
retained robust-prune behavior for spare-capacity targets nor suppresses
stale-neighbor cleanup when the current neighbor population is incomplete.
This prevents a rejected backlink from perturbing established traversal edges
without adding the reverse edge, and directly implements FR-083's
edge-preservation requirement.

The insert-work surface gains a coordinator-scoped
`backlink_prune_rejected` counter, and benchmark labels identify the candidate
and excluded append-only control truthfully. Focused product, counter, parser,
and gate-control tests pass at exact head `3da6df06c`.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
