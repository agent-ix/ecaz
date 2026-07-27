---
task: 189
packet: 001-entry-trigger
role: coder
status: open
date: 2026-07-26
head: c1c43a9bf
---

# Review request: Task 189 entry trigger

Task 189 is conditionally dormant pending the required same-seed trigger. The
prior Task 183 exact-neighbor arm did not provide one: exact scoring measured
0.9605 recall versus 0.9625 for RaBitQ and raised warm p50 from 43.8 ms to
113.1 ms. That result is recorded in
`reviews/task-183/002-codec-attribution/` and the Task 183 STOP packet.

Task 188's completed attribution and full-scale matrix point at bounded
entry/traversal budget, not neighbor-code ordering error: at 100k the owner
oracle reached 0.9970 recall but was approximately 2.49 seconds mean, while
the bounded BW4/H100 arm reached 0.9740 recall at 42.4 ms. The selected BW8
search arm reached 0.9805 at 44.1 ms mean with unchanged storage, confirming a
search-budget tradeoff rather than a codec-error trigger.

Decision at this gate: do not repeat the unchanged exact-neighbor arm. Keep
codec experimentation dormant unless Task 188's completed full-scale evidence
shows that reachable candidates are specifically lost through approximate
neighbor ordering or reports an actionable RaBitQ error margin.

Evidence and provenance are summarized in `artifacts/manifest.md`; the final
Task 188 result is recorded on branch `task-188-graph-search-residual` at
head `171b84898`.
