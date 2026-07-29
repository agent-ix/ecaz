---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 search and graph residual attribution

## Result

The fresh 100k physical generation confirms that entry coverage is not the
whole residual: the exact bounded head reaches 0.9740 while the owner oracle
reaches 0.9970 under the same BW4/H100 search budget. The same seed IDs were
held fixed across the BW/H controls.

BW2/H100 and BW4/H50 do not improve recall. BW8/H100 raises recall to 0.9805
with essentially unchanged mean latency (42.70 vs 42.40 ms), while increasing
p95 latency to 53.20 ms and remote candidate work from 25.86 to 29.56 per
scan. Storage and head bytes are unchanged. This is a useful but not yet
production-safe bounded search candidate.

## Attribution

The stage/work counters show bounded traversal and remote expansion are
material contributors, but the candidate’s gain is not attributable to a
different entry set: all exact-head BW/H variants have seed digest
`26daaa06e76426faeb151329efc77f04c9fb44684ce81600055e13955b154bcd`. Remote
and traversal reconciliation passed for every physical arm. The owner oracle
is retained only as an upper-bound attribution control.

## Decision

Advance only `bw8-h100` to the isolated candidate matrix in packet
`003-isolated-candidate`. Do not change graph construction, persisted formats,
or runtime defaults from this screen.

See [manifest.md](artifacts/manifest.md),
[task188-residual-attribution-results.log](artifacts/task188-residual-attribution-results.log),
the checked-in [entry suite config](../001-entry-and-residual-plan/artifacts/task188-residual-attribution-100k-suite.json),
and the structured [suite results](artifacts/run/results.jsonl).
