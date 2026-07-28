---
agent: codex
role: coder
model: GPT-5
date: 2026-07-27
seq: 1
---

# Task 200 attribution

The RSS growth is isolated to the benchmark-only seed-coverage diagnostic.
The production physical latency path remains flat with stage counters both off
and on, using one backend for all 300 queries. PostgreSQL’s memory-context
dump during the standalone coverage statement reached 8.32 GB of backend
memory, while the production latency backend remained near 260 MB.

No production read-path fix is warranted by this evidence. The benchmark-only
coverage helper should be bounded or documented separately from production
latency closeout; the next packet records the regression/closeout decision.

See [`artifacts/attribution-summary.md`](artifacts/attribution-summary.md) and
the cited reproduction packet for raw series and node-log evidence.
