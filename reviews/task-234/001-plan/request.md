---
task: 234
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 234 read RPC deadline and cancellation parity plan

This packet requests review of Task 234 at planning checkpoint `dd3e37078`.

FR-081's Task-214 F9 gap originally identified four distributed read/control
calls that still use bare awaits. The entry inventory found a fifth:
crown-code export. Physical head search, crown-code export, gateway-routing
export, head-shard export, and head-shard import all move under the same
nonzero client deadline, remote statement timeout, PostgreSQL interrupt,
bounded cancel-token delivery, and fail-closed aggregation contract used by
expansion and materialization.

The task also requires an explicit pooled-connection disposition after timeout
or cancellation, stops later owner dispatch after a local interrupt, and proves
that one failed owner cannot yield partial results or stale-response reuse.
Focused PG18 multinode faults cover stalled statements, local cancel/timeout,
remote backend termination, connection reset, and mixed sibling success/fail.

Please review the call inventory boundary, interrupt/cancel semantics,
connection-eviction rule, no-partial-result invariant, and fault matrix. Task
235 intentionally owns transaction-control/write calls whose ambiguous
outcomes need different recovery semantics.

This is planning-only. No tests were run.

The task-local call-site inventory is
`artifacts/remote-callsite-inventory.md`. It separates already-bounded read
paths, the five Task 234 gaps, Task 235 transaction/write awaits, and
synchronous libpq callbacks so the structural acceptance scan has an explicit
allowlist instead of relying on the original four-call statement.
