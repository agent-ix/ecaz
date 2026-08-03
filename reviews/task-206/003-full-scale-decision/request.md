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

Head SHA: `74d752ddb`

This packet opens the Task 206 closeout lane with a real 10k diagnostic
SuiteConfig at `artifacts/task206-10k-diagnostic.json`, using the canonical
`ecaz bench suite` runner, BW=32/H=8, top-k 200, and the archived 10k DBpedia
corpus. It is deliberately a short diagnostic (5 timed iterations) to verify
that the current PG18 install can complete a real-corpus physical run after
the 100k setup stall.

This is not yet the required 10k/50k/100k recall+latency+storage closeout
matrix. Results, if produced, will be added under this packet and the full
matrix will remain open until all required scales and the winning A/B are
measured.
