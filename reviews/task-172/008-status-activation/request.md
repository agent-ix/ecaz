---
task: 172
packet: 008-status-activation
role: coder
status: review-requested
head: 0fc804c7689082b4a4203bba64cae92cdcb9a4df
date: 2026-07-29
---

# Review request: activate Task 172

## Requested decision

Please review commit `0fc804c7689082b4a4203bba64cae92cdcb9a4df`,
which changes Task 172 from `SHELVED` to `IN PROGRESS` in the canonical task and
task index.

## Basis

Packet 004 established that Task 172's sole shelving condition is met:

- Task 179's physical three-owner lane exists;
- TC-040, TC-042, and TC-050 have accepted evidence; and
- the topology audit is fail-closed.

The operator then directed work to proceed on Task 172. Packets 005-007 have
already landed the first runner-capability slices.

## Boundary

This status change does not close, promote, or merge Task 172. The final
decision-bearing matrix remains open pending:

- Task 204's required storage-measurement proof;
- Task 205's fixed-regime pushdown A/B;
- Task 206's traversal-regime disposition; and
- Task 208's mechanical NFR-021/NFR-022 gates.

Task 166 remains single-instance control evidence. Task 165 remains a
replicated-serving control and cannot supply a distributed gate row.

## Validation

Documentation-only change. `git diff --check` passed; no tests or benchmarks
were run.
