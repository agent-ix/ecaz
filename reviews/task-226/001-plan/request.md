---
task: 226
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 226 current-head BW8 transfer plan

This packet requests review of the Task 226 plan at checkpoint `d42d01e32`.
The task isolates BW4/H100 versus BW8/H100 on the current conforming fixed-4096
sharded generation and tests whether Task 188's smaller simultaneous
recall/latency win survives pushdown, gateway copies, lazy materialization, and
the production head.

The contract preregisters four ordered same-generation variants:
byte-identical BW4 A/A followed by `bw4-control`/`bw8-candidate`, whose exact
names activate paired per-query recall. It separates the clean production
latency row from a fresh full-metrics attribution fixture and defines the
advance boundary before measurement: recall must be non-worse, and BW8 must
either clear the 1 ms/5% latency gate without a material tail regression or
deliver a paired recall gain inside a 5% warm-mean/p95 envelope. A larger
latency cost is a Task 219 recall/latency trade and STOPs without a full matrix.

Please review the changed-premise justification, same-generation controls,
numerical disposition rule, separate attribution surface, and exclusion of
previously answered BW64/H8 and cap-16384 head experiments. The task records a
Pareto point but does not change the production default without a separate
reviewed policy disposition.

This is a planning-only packet. No new benchmark result is under review.
