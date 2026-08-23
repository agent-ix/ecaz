---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 reject append-when-room default

Status: review requested for code checkpoint `58dfae568`. Task 167 remains
implementation-open; this is a measured-negative disposition, not closeout.

Packet 051's exact-runtime isolated 50k run rejected candidate `22c1e01c3`.
The inserted-neighborhood population passed, but the dominant 200-query
heldout population measured a `0.010611` physical-vs-fresh deficit against the
fixed `0.007000` allowance. That was `0.002000` worse than packet 047's
isolated robust-prune-only deficit.

This checkpoint restores the pre-candidate robust-prune default, updates its
GUC documentation and regression test, and restores truthful harness
attribution: robust-prune is measured and quality-gated first; the rejected
append candidate cannot mutate the fixture unless that shipped gate passes.
The generic structured backlink-strategy metric remains available. The PG18
default regression, suite-parser regression, and both quality-gate control-flow
tests pass.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
