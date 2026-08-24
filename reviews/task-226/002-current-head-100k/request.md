---
task: 226
packet: 002-current-head-100k
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 226 current-head 100k preregistration

This packet preregisters the suite configuration for the fixed-4096 current
head BW4/H100 versus BW8/H100 transfer screen. No result is claimed yet.

The production step runs `aa-control`, `aa-candidate`, `bw4-control`, and
`bw8-candidate` on one immutable generation. The A/A pair must be byte
identical; the exact BW4/BW8 names activate paired per-query recall. Only beam
width changes in the A/B. A separate fresh full-metrics fixture captures stage
and work attribution so instrumented latency is not the production decision
row.

The numerical ADVANCE/TRADE/STOP rule is recorded in the Task 226 file at
`d42d01e32`. The checked-in SuiteConfig is
`artifacts/task226-current-head-bw8-100k.json`; audit and expanded-command
evidence will be added before either long run.
