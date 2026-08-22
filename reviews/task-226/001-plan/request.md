---
task: 226
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-21
seq: 01
---

# Task 226 current-head BW8 transfer plan

This packet requests review of the Task 226 plan at planning checkpoint
`daf2b1fb1`. The task isolates BW4/H100 versus BW8/H100 on one current,
conforming fixed-4096 sharded generation. It tests whether Task 188's smaller
simultaneous recall/latency win survives pushdown, gateway copies, lazy
materialization and the current production head.

Please review the changed-premise justification, same-generation controls and
the exclusion of previously answered BW64/H8 and cap-16384 head experiments.
The task records a Pareto point but does not change the production default
without a separate reviewed policy disposition.

This is a planning-only packet. No new benchmark result is under review.
