---
task: 235
packet: 001-plan-and-phase-inventory
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 235 write and lifecycle RPC cancellation plan

This packet requests review of Task 235 at planning checkpoint `dd3e37078`.

The principal physical mutation endpoint calls are bounded, but surrounding
BEGIN/SET/PREPARE/COMMIT/ROLLBACK, intent, callback, reaper, and lifecycle
statements do not have one uniform cancellation/recovery contract. Ordinary
read-query timeout handling cannot be applied mechanically: a lost response
after prepare or commit may leave an outcome-unknown distributed transaction.

Task 235 therefore starts with a complete phase inventory and classifies each
failure as definitely not applied, definitely applied, or outcome unknown. It
preserves Task 167's durable intent/GID fences, evicts connections with
ambiguous transaction state, leaves uncertain decisions to idempotent
operator-driven recovery, and requires fault injection at every 2PC boundary
plus build/handoff/publish/retire/abort coverage where the transport is shared.

Please review the boundary with Task 167, outcome taxonomy, callback behavior,
intent durability, operator-recovery guarantees, and phase-by-phase PG18
fault matrix. The task must not infer commit/rollback from timeout or age.

This is planning-only. No tests were run.
