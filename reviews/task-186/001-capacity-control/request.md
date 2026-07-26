---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 186 capacity-control screen

## Scope

This packet measures the existing Task 186 head-capacity path only. It does
not change production code or defaults. The 100k screen compares deterministic
training-landmark caps with identical query work and physical topology.

## Result

The 8,192 cap is a useful monotonic improvement over the 4,096 control:
physical recall rises from 0.9625 to 0.9690, coverage zero-fraction falls from
1.5% to 0.5%, and warm physical latency rises from 19.70 ms to 22.40 ms.

The conditional 16,384 screen continues the recall trend to 0.9740 and zero
coverage misses, but warm physical latency rises to 27.10 ms and the estimated
head cache reaches 103.6 MB. That is a recall/latency/cache tradeoff, not a
clear production promotion. The capacity result should therefore remain a
benchmark candidate and feed the bounded compressed/hierarchical screen; no
production task is advanced from this packet.

## Evidence

See [manifest.md](artifacts/manifest.md),
[task186-capacity-control-results.log](artifacts/task186-capacity-control-results.log),
the checked-in [suite config](artifacts/task186-capacity-control-100k-suite.json),
and the structured [100k results](artifacts/run-benchmark-feature/results.jsonl).

All three arms passed topology and remote-owner engagement checks. The full
10k/50k/100k closeout matrix is intentionally not claimed here: this packet is
the permitted 100k candidate screen, and only a selected bounded design should
advance to the required full-scale matrix.
