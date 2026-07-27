---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 corrected batch-10 confirmation

The reviewer identified a benchmark-runner asymmetry: the historical Task 188
variant strings omitted materialization batch size, and the old parser mapped
that omission to eager-0 while Tasks 185/186 used production lazy-10. This
packet reruns the required BW4-control versus BW8-candidate matrix at 10k/50k/
100k with explicit batch-10 and captures paired per-query outcomes over the
same 200 queries.

## Result

BW8 remains the sole isolated search-budget candidate. It is recall-neutral at
10k, gains 0.0025 at 50k, and gains 0.0065 at 100k. The paired results are 0/0
wins at 10k, 5 candidate wins versus 0 control wins at 50k, and 7 candidate
wins versus 0 control wins at 100k. The bootstrap intervals are positive at
50k and 100k. Batch-10 latency is lower for BW8 than BW4 at every scale, with
identical physical storage and passed topology/remote-owner engagement.

This accepts BW8 as a research candidate under the task rule, without changing
production defaults, persisted formats, or opening productionization in Task
188. Any production change remains a separate task with its own gates. The
historical packet 003 rows remain useful only as explicitly labeled eager-0
unbatched evidence.

See [manifest](artifacts/manifest.md), [cited results](artifacts/task188-bw8-batch10-results.log), and the structured `ecaz bench suite` outputs under
`artifacts/run/`.
