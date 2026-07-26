---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 186 bounded hierarchy screen

## Scope

This packet measures the pre-registered two-level representative/group
candidate at a 16,384-entry training-landmark head. It changes no production
code or defaults. The physical generation, query set, search budget, and
topology are held fixed for the candidate screen.

## Result

The hierarchy is not a useful isolated candidate. Coverage is not the limiting
factor: zero-owner coverage misses are 0% and owner membership is 0.6155.
Nevertheless, final recall is 0.9440 and warm physical latency is 84.30 ms
mean / 77.60 ms p50 / 125.30 ms p95. The exact 16,384 control in packet
`001-capacity-control` reached 0.9740 recall at 27.10 ms mean with roughly
103.6 MB estimated head cache. The hierarchy therefore loses both recall and
latency despite using the same stored head capacity.

## Decision

STOP. The hierarchy does not advance to the required full-scale matrix or a
production task. Task 186’s remaining optimization result is the bounded
capacity tradeoff already recorded in packet `001-capacity-control`; no
production promotion is claimed.

## Evidence

See [manifest.md](artifacts/manifest.md),
[task186-hierarchy-screen-results.log](artifacts/task186-hierarchy-screen-results.log),
the checked-in [suite config](artifacts/task186-hierarchy-100k-suite.json), and
the structured [suite results](artifacts/run/results.jsonl).

Topology, remote-owner engagement, and deterministic provenance checks passed.
The raw operational logs remain uncommitted per the repository review-packet
rules.
