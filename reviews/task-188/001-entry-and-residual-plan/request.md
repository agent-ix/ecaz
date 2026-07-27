---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 entry and residual plan

## Entry decision

Task 185’s fixed-cap gateway screen stopped: its arm selected only 127 positive
landmarks and filled the remaining 3,969 slots from the control, so its
Jaccard-1 result is structurally constrained rather than a family-wide gateway
refutation. Task 186 then screened bounded capacity. The exact 16,384 head
reached 0.9740 recall at 100k. The two-level hierarchy retained zero
owner-coverage misses but fell to 0.9440 recall at 84.30 ms mean; that result
is limited to its query-time/arbitrary-representative prototype, which
re-buckets the stored head per query. The compressed head was not screened.
These qualified results satisfy entry for a residual search-budget screen
without claiming that all head families are exhausted.

The Task 186 packets cited by this entry were on the separate, unmerged
`task-186-bounded-hierarchical-head` branch and had an open prototype-scoped
review. They are inherited context, not a merged production conclusion. The
retained exact-scored 16,384 training-landmark head is consequently an
experimental reference surface only; Task 188 opens no promotion path for it.

This satisfies Task 188’s conditional entry gate. The residual experiment is
pre-registered against the best bounded head that actually remains viable:
the exact-scored 16,384 training-landmark head, with the hierarchy excluded
only as a measured prototype, not rejected as a whole family.

## Fresh Phase 1 matrix

One fresh 100k physical generation will compare, under identical graph and
query fixtures:

- `bounded-head`: exact-scored bounded-head seeds, BW4/H100;
- `owner-oracle`: owner-scan seeds, BW4/H100;
- `bw2-h100` and `bw8-h100`: isolated BW controls;
- `bw4-h50`: isolated H control.

All arms use 32 head seeds, RabitQ neighbor scoring, top-k 10, 200 held-out
evaluation queries, warm serial latency, and the same three-owner physical
topology. The suite enables DistANN stage counters so query work is attributed
by head scoring, local/remote expansion, traversal, and materialization. The
owner-oracle arm is an attribution control, not a bounded production
candidate.

## Candidate rule

No graph or adaptive-search change is selected from the evaluation result. A
single candidate may be pre-registered only after the isolated controls show a
dominant residual family and only if it improves recall without an unacceptable
latency, storage, build, remote-work, or topology tradeoff. Otherwise the task
records STOP and hands any ordering-specific residual to Task 189.

## Evidence

The checked-in suite config is the pre-registered Phase 1 matrix. Results will
be added to packet `002-search-graph-attribution` under the packet-local
artifact manifest; raw corpus and PostgreSQL operational logs will not be
committed.

The historical Phase 1 run completed only the head-vs-owner oracle, isolated
BW, and isolated H comparisons. Candidate-frontier/exact-rerank containment,
graph components/indegree/bridge/hard-query reachability, and
monolithic-versus-sharded graph-quality audits were not run. Its result is
therefore a search-budget screen, not attribution of the full graph family.
