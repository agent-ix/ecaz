---
task: 221
packet: 001-preregistration-and-screen
agent: Codex
role: coder
model: gpt-5
date: 2026-08-10
seq: 01
---

# Task 221 MAT-22 preregistration

This packet preregisters the isolated 100k screen for MAT-22: carry the owner
row-tier locator from physical expansion through the coordinator and use it at
owner payload materialization, eliminating the owner directory lookup. The
candidate is benchmark-feature/GUC gated; production endpoints remain on the
existing path.

The control and candidate use the same persisted-head search, lazy-10 window,
projection, payload representation, and immutable physical generation. The
suite requires same-generation recall pairing and materialization correctness.

Evidence and the decision will be added under this packet's `artifacts/` after
the `ecaz bench suite` run. A neutral or regressing isolated result is a
pre-registered STOP; no 10k/50k matrix is authorized unless the 100k screen is
useful and preserves recall, ordering/prediction identity, storage, and the
NFR gates.

- config: `artifacts/task221-mat22-100k.json`
- manifest: `artifacts/manifest.md`
- implementation checkpoint: `0b6a4bbbf`
