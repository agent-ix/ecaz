---
task: 187
packet: 004-full-scale-decision
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Full-scale decision

Task 187 closes on the STOP branch. No production code or quantizer/index
behavior changed, so no new 10k/50k/100k candidate matrix is warranted. The
accepted Task 191 packet 003 remains the full-scale production lazy10 evidence;
this task adds the fresh 100k traversal attribution and records that remote
owner transport is the next investigation target.

Next task: add traversal-transport observability (per-owner request encode,
wait, decode, and straggler attribution) and then evaluate exactly one bounded
transport optimization with the standard 10k/50k/100k A/B suite.
