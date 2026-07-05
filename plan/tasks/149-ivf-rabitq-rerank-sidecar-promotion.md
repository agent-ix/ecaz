# Task 149: IVF RaBitQ rerank sidecar promotion (Task 51 follow-up #1, realized on 111g/h machinery)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P1

## Why

Task 51's closeout (`reviews/task-51/023-round-closeout/request.md`) established
that high-recall IVF RaBitQ at 1m is structurally heap-fetch + toast bound
(~18 ms of ~33 ms/query at nprobe=128 on `real[]`), invisible to every scoring,
geometry, or layout lever. Its follow-up #1 — a `rabitq8` or f16 TID-sorted
rerank sidecar — was the round's best measured Pareto result (total-bound p50
**43.5 ms vs 69.1 ms baseline**, ~37% projected, at 1.43 GiB) and was **never
filed as a task**. Since then, Task 111g/h landed the persisted index-side
rerank machinery (compact `0x2A` sidecar, `rerank_placement='index'`,
`rerank_format` ∈ {f16, rabitq4, rabitq8, turboquant}), so the plumbing now
exists; 111h's closeout promoted `source/f32` as default and left index-side
compact formats "iterate via new tasks". This is that task, aimed squarely at
the Task 51 heap-fetch finding.

## Scope

- A/B `rerank_placement='index'` + `rerank_format` ∈ {f16, rabitq8} against the
  `source/f32` default on the IVF RaBitQ lane (quantizer='rabitq', the staged
  bit-widths in production use), 10k/50k/100k first, escalate to 1m (where the
  heap-fetch bound actually bites) if smaller scales are neutral-or-better.
- Measure the tail risks Task 51 recorded before any promotion: random-ID p99
  blowup (529 ms datum) and TID-sorted concurrency c4 p99 (335 ms datum).
- Recommendation: promote / keep opt-in / reject, with the recall+latency+storage
  tables per NFR-007.

## Out of Scope (hard)

- No new on-disk format — the 111g/h `0x2A` sidecar is the vehicle. If it is
  missing an option this task needs, extend it as its own reviewed slice first.
- No default flip without the full 10k/50k/100k(/1m) matrix.

## Gate / Exit Criteria

- `ecaz bench suite` A/B evidence at 10k/50k/100k (1m if promoted) for
  source/f32 vs index/f16 vs index/rabitq8 on the RaBitQ lane, recall +
  latency + storage, tail latency included; a promote/keep/reject decision
  recorded in the packet. Closes on the decision, either way.
