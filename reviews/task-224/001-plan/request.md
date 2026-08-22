---
task: 224
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-21
seq: 01
---

# Task 224 owner payload locality plan

This packet requests review of the Task 224 plan at planning checkpoint
`daf2b1fb1`. The task is conditional on the post-222/223 profile and begins
with heap-block dispersion, cache, TOAST, detoast and binary-send attribution.
It advances at most one of TID/block reorder or block-batched detoast only when
the addressable physical-locality ceiling is at least 1 ms/scan or 5% of warm
100k mean.

Please review the attribution requirements, exact rank-restoration contract and
the prohibition on combining locality work with a row-tier format change.

This is a planning-only packet. No implementation or benchmark result is under
review.
