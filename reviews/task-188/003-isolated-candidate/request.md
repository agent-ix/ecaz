---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 isolated BW8 candidate

Phase 1 selected only BW8/H100 for confirmation as a search-budget candidate.
It held the exact bounded head seeds constant and improved 100k recall from
0.9740 to 0.9805. The historical full-scale rows omitted the materialization
batch field, which the old parser mapped to eager-0; those latencies remain
historical unbatched evidence and are not acceptance measurements. This packet
therefore pre-registers a corrected A/B confirmation at 10k, 50k, and 100k
with explicit production batch-10, measuring recall, paired per-query
wins/losses, warm latency, storage, build, head bytes, topology, and engagement
for BW4 control versus BW8 candidate.

The suite keeps graph degree, head policy/cap, neighbor code format, seed
width/count, topology, and query fixture fixed within each scale. It does not
combine a graph rebuild with an adaptive policy and does not alter production
defaults.

The historical unbatched full-scale suite is not a final acceptance decision.
The corrected batch-10 confirmation is recorded in packet
[`005-batch10-reconfirmation`](../005-batch10-reconfirmation/request.md). The
final decision must use its paired recall and latency/storage/build rows; it
must either accept BW8 now or STOP, not defer the decision to a follow-up task.

See `artifacts/task188-bw8-full-scale-results.log` for the cited result lines
and `artifacts/run/results.jsonl` for the structured suite source of truth.
