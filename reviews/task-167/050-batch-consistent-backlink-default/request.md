---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 batch-consistent backlink default

Status: review requested for code checkpoint `22c1e01c3`. The checkpoint is a
benchmark candidate, not an accepted Task 167 quality or closeout result.

Packet 047 removed the rejected-arm contamination but left a clean heldout
deficit `0.001611` outside the fixed band. Static reconciliation found that the
shipped physical path robust-pruned a backlink target even while it had spare
degree. That behavior contradicts both reference surfaces already in this
tree:

- batch Vamana appends a backlink while `out_degree < max_degree` and only
  robust-prunes a full target; and
- the shared Task 167 pure planner documents and tests the same invariant:
  free capacity appends without removing existing edges.

The earlier throughput disposition had changed the GUC default so every target
followed robust-prune union. That can remove existing edges before capacity is
reached, a graph mutation the fresh batch reference never performs.

This checkpoint restores append-when-room as the production default, retains
robust-prune-all behind the existing diagnostic GUC, and changes the harness so
the shipped arm is quality-gated before that diagnostic control can mutate the
fixture. The fixed packet-045 quality bands are unchanged. A clean 50k
before/after comparison against packet 047 is required before promotion, then
the full isolated 10k/50k/100k confirmation remains required.

The extension default test, shared planner test, suite-parser test, and quality
gate control-flow tests pass. Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
