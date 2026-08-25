---
task: 228
packet: 002-post-layout-hardening-resequence
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 228 post-layout and transport-hardening resequence

This packet requests review of the Task 228 plan update at checkpoint
`dd3e37078`.

The prior plan ran after Tasks 222--227. Tasks 229--233 now introduce mandatory
storage prototypes that can change owner service, bytes, cache residency, build
cost, and the end-to-end denominator. Tasks 234--237 close production transport
gaps in deadline/cancel parity, distributed transaction cleanup, TLS/secret
handling, stable errors, and EXPLAIN metrics. Task 228 now runs after those
tasks so its real-network and BatANN trigger decision measures the selected
production-eligible surface.

The matrix is broadened to report head/traversal/materialization/maintenance
message counts, encoded bytes including PostgreSQL framing where measurable,
owner service versus socket wait, concurrency 1/2/4/8/16, pool opens/reuse/
evictions, backpressure, queueing, and owner saturation. The ADR-085 D4 >=50%
transport-share trigger and the restriction that GO authorizes only new
specification work remain unchanged.

Please review the dependency ordering, whether Tasks 234--237 are sufficient
to make the transport substrate production-eligible, and whether the broadened
matrix is adequate to trigger or reject BatANN, multiplexing, or wire work.

This is planning-only. No tests or benchmarks were run.


