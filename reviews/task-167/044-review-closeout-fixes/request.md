---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 review-closeout fixes

Status: review-open for code checkpoint `c5d7d8041`; the preregistered repeat
measurements and variance-derived quality gate remain open and will land in a
subsequent packet. No Task 167 closeout is claimed here.

Please review checkpoint `c5d7d8041`, which processes every code-level finding
from packet 043 feedback:

- The two recall readings are now labeled and calibrated at their actual graph
  states. Ordinary `ecaz bench recall` runs before incremental DML; the exact
  physical-vs-fresh comparison runs after 320 physical inserts. Before those
  inserts, the harness feeds the ordinary arm's exact truth and predictions
  through the exact instrument's distinct scorer and fails unless the two
  readings agree within the ordinary table's half-unit display precision.
- Post-insert heldout quality now uses all 200 ordinary heldout queries plus a
  separate fixed 48-query inserted-neighborhood population. This removes the
  old 152-query subset and makes the before/after graph-state comparison
  auditable on the same heldout set.
- `task167_ann_predictions` receives the complete production operating point:
  beam, heap, hops, top-k, caller session GUCs, seed controls,
  materialization/locator controls, head placement, replica, gateway, and crown
  controls. The emitted rows attest `search_gucs_pinned=true`.
- Append-when-room is dispositioned off by default. The existing diagnostic
  GUC still enables the rejected candidate arm explicitly, then the harness
  resets to the shipped robust-prune path. This follows the completed
  10k/50k/100k A/B, where enabled/disabled throughput was
  `0.975741 / 0.997529 / 0.993053` and never won.
- The misleading `rank_insert_candidates` self-drop comment now describes the
  actual caller-owned exclusion contract.

Focused validation is green: 10 Task 167 CLI tests pass, including a new test
that enumerates the pinned production search settings, and the PG18 extension
compile check passes. Packet-local logs and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).

The remaining reviewer requests are intentionally not claimed by this packet:
3–5 preregistered 10k repeats, a repeated 50k heldout measurement, derivation
of a run-variance quality band, restoration of the hard automated gate, and
final status bookkeeping after outside review.
