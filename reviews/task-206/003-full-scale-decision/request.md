---
task: 206
packet: 003-full-scale-decision
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 003
---

# Review request: full-scale decision lane opened

Code head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

This packet opens the Task 206 closeout lane with a real 10k diagnostic
SuiteConfig at `artifacts/task206-10k-diagnostic.json`, using the canonical
`ecaz bench suite` runner, BW=32/H=8, top-k 200, and the archived 10k DBpedia
corpus. It is deliberately a short diagnostic (5 timed iterations) to verify
that the current PG18 install can complete a real-corpus physical run after
the 100k setup stall.

The run completed the 10k diagnostic. The physical arm measured recall 0.9526,
p50 latency 172.0 ms (5 timed queries), and 242,745,344 physical-generation
bytes; the single-index control measured recall 0.8971 and p50 latency 32.4
ms. The packet-local `results.jsonl` and summary log are the durable source of
truth.

This is not yet the required 10k/50k/100k recall+latency+storage closeout
matrix: 50k and 100k result rows are still open, and the 100k setup attempt is
recorded separately as stalled before metrics.

`artifacts/task206-50k-diagnostic.json` is now pre-registered for the next
real-corpus scale using the same BW32/H8 diagnostic shape.
