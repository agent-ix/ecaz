---
task: 226
packet: 003-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 226 full-scale BW8 confirmation preregistration

Packet 002's current-production 100k screen satisfied Task 226's registered
recall-improvement ADVANCE branch. BW4/H100 to BW8/H100 moved paired recall
from 0.9285 to 0.9450 (delta +0.0165, bootstrap 95% CI +0.0080 to +0.0265),
warm mean from 16.4 to 16.2 ms, and p95 from 19.0 to 19.8 ms (+4.21%). The A/A
predictions were byte-identical, topology/storage conformed, and only beam
width changed.

This packet preregisters the remaining fresh production confirmations at 10k
and 50k. Together with packet 002's immutable 100k production result, they are
the required 10k/50k/100k matrix; the 100k build is not repeated. Each new
scale uses one fresh three-owner generation shared by `bw4-control` and
`bw8-candidate`. Fixed inputs are the 4,096 persisted sharded head, L32, H100,
RaBitQ, lazy-10 materialization, query set, topology, graph degree, and storage
format. Only beam width changes from 4 to 8.

At each scale the Task 226 rule is applied literally: paired-recall point and
bootstrap lower bound must be nonnegative, then either the primary latency-win
branch or the recall-improvement-within-5%-mean/p95 branch must pass. Any scale
outside those branches prevents a production-default recommendation. Storage
and topology must conform at every scale. No default change is made by this
packet; final policy remains review-gated.

Please review the preregistered config and, when results land, the per-scale
paired recall, warm latency/tails, storage, topology, engagement, release SHA,
and the final all-scale disposition.
