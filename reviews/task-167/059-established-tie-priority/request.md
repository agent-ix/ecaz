---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 established-backlink tie priority

Status: review requested for candidate checkpoints `350385ce9` and
`ddea621a6`. No quality, scale-matrix, merge, or closeout result is claimed by
this code packet.

Packet 057 falsified the prior full-target no-op hypothesis: preserving 702
prune-rejected targets moved heldout recall by exactly zero versus packet 054.
The follow-up audit found a different physical/local mismatch. The physical
backlink path assembled its robust-prune union with the proposed backlink
first, while the pure DistANN planner, mature local incremental planner, and
batch Vamana full-target path place established neighbors first.

`robust_prune` breaks exact-distance ties by candidate ordinal. That ordering
is observable in the Task 167 fixture because every inserted vector is an
exact duplicate of an existing corpus vector. When a target already links to
the original vector, proposal-first ordering lets the new duplicate win the
tie and can evict the established edge solely because of temporary union
position.

This candidate centralizes backlink-union construction with established
neighbors first and routes both the pure and physical planners through it. A
regression proves that proposal-first selection chooses the new duplicate,
while the corrected backlink helper retains the established neighbor. Harness
labels identify the candidate and excluded append-when-room control without
changing thresholds or benchmark logic.

Focused product, parser, and quality-gate controls pass. Validation and
provenance are in [`artifacts/manifest.md`](artifacts/manifest.md).
