---
task: 223
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-21
seq: 01
---

# Task 223 direct owner tuple materialization plan

This packet requests review of the Task 223 plan at planning checkpoint
`daf2b1fb1`. The task is gated on Task 222 and first decomposes the refreshed
owner cost into tuple fetch, detoast/send, SPI/executor, array construction and
response assembly. Direct tuple access is authorized only when the addressable
residual is at least 1 ms/scan or 5% of warm end-to-end mean at 100k.

Please review whether the proposed relation-backed tuple-slot and cached
binary-send candidate preserves snapshot, ordering, null, TOAST, schema,
tombstone and failure semantics. This is intentionally distinct from Task
220's rejected SQL concatenation arm.

This is a planning-only packet. No implementation or benchmark result is under
review.


