---
task: 207
packet: 003-full-scale
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 003
---

# Review request: full-scale partition-union A/B

Code head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

This packet registers the required 50k and 100k Task 207 control/candidate A/B
through `ecaz bench suite`. It holds BW=128, H=5, head cap 4096, top-k 200,
and seed count 200 fixed while comparing `build_shards=1` with the
`build_shards=4` per-partition BFS-prefix union. The full-scale arms use the
persisted-head production strategy so the construction change is isolated.
The owner-scan oracle remains covered by the 10k diagnostic; a full-scale
owner-scan attempt was stopped after it produced no result artifact because
the exact owner scan makes the 50k latency matrix impractical.

The 10k diagnostic already recorded a candidate recall increase from 0.9529 to
0.9615 and a physical storage increase from 242,745,344 to 244,285,440 bytes.
The completed 50k A/B raised recall from 0.8814 to 0.8994, reduced p50 latency
from 333.0 ms to 323.6 ms, and increased physical generation storage by
2,506,752 bytes. See `artifacts/result-summary-50k.md` and
`artifacts/run-50k-final/results.jsonl`. The 100k result remains open until
its suite artifacts land.

Please review the preregistration and leave findings under this packet's
`feedback/` directory.
