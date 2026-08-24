---
task: 226
packet: 003-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 226 full-scale BW8 confirmation decision

Packet 002's current-production 100k screen satisfied Task 226's registered
recall-improvement ADVANCE branch. BW4/H100 to BW8/H100 moved paired recall
from 0.9285 to 0.9450 (delta +0.0165, bootstrap 95% CI +0.0080 to +0.0265),
warm mean from 16.4 to 16.2 ms, and p95 from 19.0 to 19.8 ms (+4.21%). The A/A
predictions were byte-identical, topology/storage conformed, and only beam
width changed.

This packet preregistered and now contains the completed fresh production
confirmations at 10k and 50k. Together with packet 002's immutable 100k
production result, they form the required 10k/50k/100k matrix; the 100k build
was not repeated. Each new
scale uses one fresh three-owner generation shared by `bw4-control` and
`bw8-candidate`. Fixed inputs are the 4,096 persisted sharded head, L32, H100,
RaBitQ, lazy-10 materialization, query set, topology, graph degree, and storage
format. Only beam width changes from 4 to 8.

At each scale the Task 226 rule was applied literally: paired-recall point and
bootstrap lower bound must be nonnegative, then either the primary latency-win
branch or the recall-improvement-within-5%-mean/p95 branch must pass. Any scale
outside those branches prevents a production-default recommendation. Storage
and topology must conform at every scale. No default change is made by this
packet; final policy remains review-gated.

All scales pass the registered rule:

- 10k: recall ties at 0.9990; mean improves 14.80 to 14.20 ms and p95 improves
  17.90 to 16.80 ms — branch (a).
- 50k: recall improves 0.9540 to 0.9690; paired delta +0.015000 with 95% CI
  `[+0.006500, +0.026000]`; mean improves 16.90 to 16.80 ms and p95 regresses
  4.12% to 20.20 ms — branch (b).
- 100k (packet 002): recall improves 0.9285 to 0.9450; paired delta +0.016500
  with 95% CI `[+0.008000, +0.026500]`; mean improves 16.40 to 16.20 ms and
  p95 regresses 4.21% to 19.80 ms — branch (b).

The disposition is `USEFUL CANDIDATE — POLICY REVIEW`, not an automatic
default change. Although the registered gates pass, p99 regresses 7.14% at
50k and 5.08% at 100k. Please review the preregistered arithmetic,
same-generation provenance, storage/topology conformance, and whether that
tail tradeoff is acceptable for the interactive default. The compact evidence
index is `artifacts/decision-summary.md`.
