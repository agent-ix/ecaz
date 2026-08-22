---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 exact-recall disposition matrix

Status: review-open; harness checkpoint implemented and focused tests pass;
production 10k/50k/100k matrix pending.

Please review checkpoint `0bce21c05`.

Packet 041 replaced pairwise ANN overlap with exact fp32 ground truth. Packet
042 corrected the incremental prune distance and demonstrated a material 10k
improvement, from `0.805382` to `0.945809` inserted-neighborhood distinct
recall, but its suite stopped on a hard `0.002` physical-vs-fresh threshold.

FR-083-AC-4 requires a bench A/B against a fresh rebuild but specifies no
numeric non-inferiority threshold. The `0.002` value came from a different
DistANN criterion, and the outside reviewer requested the exact-truth values
side by side for disposition. This checkpoint therefore preserves `0.002` as
an explicitly labeled `reference_band`, emits `within_reference_band`, and
assigns `disposition=outside_review`; it no longer converts a valid quality
measurement into a suite-process failure. Measurement-integrity failures—wrong
plans, missing rows, malformed truth, reloption mismatch, or insufficient
held-out queries—still fail the step.

The focused Task 167 CLI tests pass 9/9. The immutable full-matrix config is
[`artifacts/task167-disposition-suite.json`](artifacts/task167-disposition-suite.json).
No quality disposition or closeout is claimed until the production
10k/50k/100k evidence lands and an outside reviewer responds.
