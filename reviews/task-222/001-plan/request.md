---
task: 222
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-21
seq: 01
---

# Task 222 qual-aware payload projection plan

This packet requests review of the Task 222 plan at planning checkpoint
`daf2b1fb1`. The task makes projection narrowing the first post-221 latency
candidate because the production id-only lane currently ships four columns,
123,076.8 payload bytes, and 8.752 ms of owner payload SQL per scan for only
6.66 remote rows.

Please review the attribute-use proof boundary: target list, quals, recheck and
other executor-visible expressions must be complete, while whole-row,
system-column, and unsupported shapes fail closed to current all-column
shipping. Also review the isolated 100k STOP gate and the requirement that only
a useful, correct candidate advances to the 10k/50k/100k suite matrix.

This is a planning-only packet. No implementation or benchmark result is under
review.
