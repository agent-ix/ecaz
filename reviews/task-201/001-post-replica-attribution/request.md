---
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 1
---

# Review request: fresh post-replica attribution

Please review the fresh 100k PG18 release-profile attribution for Task 201. The packet uses the accepted Task 199 normal-replica control: persisted trained 4096 head, BW4/H100, RaBitQ neighbor scoring, exact ranking, lazy10 materialization, and no owner payload plan cache.

The A/B is valid and seed-matched:

| arm | recall | mean | p95 | traversal total | remote materialize | storage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| owner fallback control | 0.9625 | 19.70 ms | 22.60 ms | 7.543 ms | 7.046 ms | 3,188,056,064 B |
| normal replica | 0.9625 | 17.40 ms | 21.40 ms | 3.615 ms | 7.474 ms | 3,188,056,064 B |

The replica saves 2.30 ms mean (11.7%) without recall, remote-row, payload-byte, or storage movement. Attribution identifies owner payload SQL/materialization as the largest remaining residual. The three-candidate screen and the single advanced candidate (MAT-40) are recorded in `artifacts/manifest.md`; packet 002 contains its isolated A/B.

Evidence:

- [`manifest.md`](artifacts/manifest.md)
- [`attribution-key-lines.log`](artifacts/attribution-key-lines.log)
- [`results.jsonl`](artifacts/run/results.jsonl)
- [`task201-attribution-100k.json`](artifacts/task201-attribution-100k.json)

The run passed release preflight and materialization correctness, including mixed local/remote rows, external TOAST, null payload, and failure-path scenarios. No failed-replica work is included in the measured latency arm; `replica_fallbacks=0`.
