---
task: 231
packet: 002-hybrid-handoff
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 03
---

# Task 231 → Task 233 hybrid handoff contract

This packet requests review of the downstream handoff at checkpoint
`4b7256f61`. It is planning-only.

Task 231 owns exactly one optional graph/vector layout selector, its persisted
descriptor, dense-ordinal directory, raw node relation, read path, lifecycle,
append-only DML overlay, telemetry, and isolated decision evidence. It keeps
the ordinary row heap authoritative for source payloads and does not read or
write Task-232 packed payload columns.

The opt-in implementation and all format fixtures remain available after Task
231 regardless of PROMOTE/STOP. Task 232 then runs independently with the
current graph heap. Task 233 is the first owner allowed to enable both selectors
and must resolve these integration questions explicitly:

- one shared owner-local dense ordinal and publication fence;
- one authoritative copy of every non-vector payload field;
- whether Task 231's `row_tid` locator is replaced by, or versioned alongside,
  Task 232's payload-generation locator;
- atomic append/publication across node and payload extents;
- a same-run four-arm graph-layout × payload-layout factorial, not a comparison
  assembled from Tasks 231 and 232's separate runs.

Task 231 will not pre-compose, predict, or optimize that interaction. Please
review whether this leaves Task 233 enough durable surface while preserving the
isolated attribution of both constituent experiments.
