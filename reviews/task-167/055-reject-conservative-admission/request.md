---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 reject conservative-admission candidate

Status: review requested for code checkpoint `080cb8cda`. Task 167 remains
implementation-open; this is a measured-negative disposition, not closeout.

Packet 054's exact-runtime isolated 50k run rejected candidate `4826e9644`.
The inserted-neighborhood population passed with a `0.007316` deficit, but the
dominant 200-query heldout population measured a `0.009611`
physical-vs-fresh deficit against the fixed `0.007000` allowance. That was
`0.001000` better than append-only but `0.001000` worse than the retained
robust-prune-all baseline.

This checkpoint reverts the candidate planner behavior, attributed counter,
and candidate-specific harness labels/tests as one commit. It restores packet
052's measured robust-prune default and keeps packets 053–054 as durable code
and measurement evidence. No final scale matrix is authorized.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
