---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 diagnostic instrumentation isolation

Status: review-open; addresses packet 039 feedback sections 5 and 7.3.

Please review checkpoint `44de4a131`.

`ec_distann_insert_work_reset()` and `ec_distann_stage_scoring_reset()` now
reset disjoint counter families. Focused extension coverage proves each reset
preserves the other family.

Retry attribution is now guarded by the off-by-default userset GUC
`ec_distann.debug_retry_attribution`; the fixture-relation existence check
remains a second guard. The concurrency drill explicitly enables the GUC on
its coordinator sessions and through each remote roster connection option.
An unrelated production table with the diagnostic name can no longer enable
writes by itself.

Insert-work artifacts now label their actual scope as
`coordinator_backend`, with `remote_owner_work_included=false`. The counters
remain useful for the coordinator-side attempt and bound assertions but no
longer imply cluster-wide aggregation.

No benchmark result is claimed by this packet. The exact-ground-truth quality
instrument and matrix rerun remain separate follow-ups.
