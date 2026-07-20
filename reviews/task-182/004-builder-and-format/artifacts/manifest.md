# Task 182 builder/format manifest

- Head under validation: `43b3ace1ae6b85ec088b84b98ed0195255c83a0f`
- Task bucket / packet: `reviews/task-182/004-builder-and-format/`
- Lane: PG18 production trained-head builder/format/read checkpoint
- Selected source decision: `reviews/task-181/006-decision-correction/`
- Selected measurement evidence: `reviews/task-181/005-full-scale-decision/artifacts/full-scale/results.jsonl`
- Storage/format: existing physical generation plus versioned head-policy and
  training-input metadata; exact persisted landmark vectors remain `real[]`
- Rerank/neighbor mode: exact head scoring; RaBitQ graph-neighbor traversal
- Validation artifact: `validation.log`
- Commands and key results: recorded in `validation.log`
- Timestamp: 2026-07-16 America/Los_Angeles
- Isolation: focused unit/compile/one-index local PG18 lifecycle validation; no
  benchmark measurement

The code checkpoint passes the focused format/determinism tests and the PG18
production lifecycle test. No recall, latency, or storage result is claimed
here. Corpus/query data is not committed.
