---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 shipped-only 50k quality diagnosis

Status: measurement preregistered; results pending. No Task 167 acceptance or
closeout is claimed.

Packet 045 reproduced a 50k heldout deficit of `0.026250`, but the measured
physical graph contained both the shipped robust-prune inserts and a later
rejected append-when-room diagnostic arm. Code checkpoint `c3b01290b` changes
the measurement order so exact quality is evaluated after only the 160 shipped
inserts and before the candidate is allowed to mutate the fixture.

This packet preregisters one isolated `ec_real_50k` PG18 fixture with the same
production operating point as packet 045: three owners, degree 32, head cap
4096, beam 4, heap 32, 100 hops, 200 heldout queries, 48 separate
inserted-neighborhood queries, exact fp32 truth, and pinned search GUCs.

The hard bands are fixed from packet 045 before this result is observed:

- inserted-neighborhood maximum deficit: `0.015`;
- heldout maximum deficit: `0.007`.

The command must fail if either shipped-only population exceeds its band. The
threshold will not be widened from this run. If the step passes, a separate
10k/50k/100k final confirmation is required before closeout. If it fails, Task
167 remains open for robust-prune insertion diagnosis.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
