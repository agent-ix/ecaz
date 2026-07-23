---
task: 190
packet: 003-adr-and-task-decision
role: coder
date: 2026-07-23
status: review_requested
---

# Review request: ADR-086 and Task 198 decision

Task 190 selects one architecture direction without implementing it:
ADR-086's fingerprint-bound coordinator traversal replica.

Task 198 is the separately numbered implementation/measurement task. It starts
feature-gated, proves a faithful same-generation local traversal, adds atomic
lifecycle and mutation-invalidating fallback, and runs one paired 100k A/B.
Only a useful result advances to the mandatory 10k/50k/100k matrix and a later
production-promotion decision.

The owner generation remains authoritative, final lazy10 payloads remain
owner-side, and the existing traversal path remains the correctness fallback.
No Task 185/186/188/189 recall decision is made here.

Please review the ADR's correctness/operations/storage/rollback contract and
whether Task 198's phases prevent an architecture prototype from becoming an
ungated production default.
