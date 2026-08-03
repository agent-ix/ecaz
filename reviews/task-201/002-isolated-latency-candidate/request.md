---
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 1
---

# Review request: isolated MAT-40 candidate

Packet 001 identified owner payload SQL/materialization as the largest residual after normal-replica traversal. This packet advances exactly one preregistered candidate: MAT-40, the projection-shape payload cache/prepared portal toggle (`owner_payload_plan_cache`). The head, graph, neighbor codec, replica lifecycle, lazy10 window, and query seeds are unchanged.

The fresh 100k A/B is:

| arm | recall | mean | p50 | p95 | p99 | physical storage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| normal replica, plan cache off | 0.9625 | 16.50 ms | 16.60 ms | 19.00 ms | 20.00 ms | 3,188,056,064 B |
| normal replica, plan cache on | 0.9625 | 16.00 ms | 15.80 ms | 18.90 ms | 19.60 ms | 3,188,056,064 B |

The 3.0% mean improvement and unchanged remote rows/payload bytes are sufficient to advance MAT-40 to the full 10k/50k/100k release matrix, but not to promote it. Review the packet-local structured results and correctness lines before treating the candidate as useful.

Evidence:

- [`manifest.md`](artifacts/manifest.md)
- [`distann-multinode-summary.log`](artifacts/run-fresh/mat40-owner-plan-cache-100k/distann-multinode-summary.log)
- [`results.jsonl`](artifacts/run-fresh/results.jsonl)
- [`task201-mat40-100k-reuse.json`](artifacts/task201-mat40-100k-reuse.json)
