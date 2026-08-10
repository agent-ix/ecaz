---
task: 221
packet: 002-isolated-candidate
agent: Codex
role: coder
model: gpt-5
date: 2026-08-10
seq: 01
---

# Task 221 MAT-22 isolated candidate result

The preregistered 100k physical PG18 A/B screen completed on one immutable
generation with the production lazy-10 window. Materialization correctness
passed all scenarios, recall was unchanged, and the control/candidate
prediction files were byte-identical. The candidate did eliminate owner node
lookup work, but the end-to-end and custom-scan latency results regressed
slightly, so the preregistered rule requires STOP and no 10k/50k/100k matrix.

The review evidence is in `artifacts/decision.md`, `artifacts/correctness.md`,
and the structured suite output `artifacts/results.jsonl`. The temporary
source run was cleaned after its decision-grade outputs were copied into this
packet.

- implementation checkpoint: `0b6a4bbbf` (extension); CLI runner checkpoint:
  `d1bd2a3bf`
- suite config: `../001-preregistration-and-screen/artifacts/task221-mat22-100k-background.json`
- suite manifest: `artifacts/suite-manifest.json`
- decision: STOP; candidate not useful at the 100k gate
