---
task: 222
packet: 003-isolated-100k
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 222 isolated 100k payload-projection result

Please review the preregistered same-generation PG18 100k A/B at source head
`c9f79be4a`. The suite completed successfully with one production-generation
delta: the control forced historical all-column payload shipping and the
candidate enabled the exact/fail-closed payload mask.

The candidate clears the 1.0 ms-or-5% advancement gate decisively. Warm mean
fell from 17.1 to 10.7 ms/scan (-6.4 ms, -37.43%); p95 fell from 19.8 to 12.8
ms and p99 from 20.4 to 12.9 ms. Recall is 0.9265 in both arms and the complete
prediction artifacts are byte-identical. The observed standard-query mask is
genuinely id-only: requested payload columns fell from 33.3 to 6.66 per scan
(five columns to one across the same 6.66 remote rows), and payload bytes fell
from 167,404.76 to 66.6 per scan (-99.9602%). Storage is unchanged.

All nine materialization scenarios pass, including null/toasted payloads,
first- and multi-window qual rejection, mixed local/remote rows, and an
injected post-first-batch remote failure. Focused executor coverage for cached
and generic plans, changed Params, LATERAL rescans, and EPQ is in packet 002.

The result authorizes and has started packet 004's full 10k/50k/100k matrix.
Detailed values and durable artifact routing are in `artifacts/decision.md`
and `artifacts/manifest.md`.
