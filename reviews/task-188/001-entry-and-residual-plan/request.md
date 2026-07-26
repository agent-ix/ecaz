---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 entry and residual plan

## Entry decision

Task 185’s fixed-cap gateway screen stopped: the gateway policy did not
improve membership recall and added latency. Task 186 then screened bounded
capacity. The exact 16,384 head reached 0.9740 recall at 100k, while the
two-level hierarchy retained zero owner-coverage misses but fell to 0.9440
recall at 84.30 ms mean. The remaining gap to the owner oracle (0.9970 in
Task 183) is therefore not closed by the retained bounded head, and the
hierarchy result does not justify further head work.

This satisfies Task 188’s conditional entry gate. The residual experiment is
pre-registered against the best bounded head that actually remains viable:
the exact-scored 16,384 training-landmark head, with the hierarchy excluded by
its measured STOP result.

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
