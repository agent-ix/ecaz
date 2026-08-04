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

This packet registers the required 50k Task 207 control/candidate A/B through
`ecaz bench suite`. It holds BW=128, H=5, head cap 4096, top-k 200, and seed
count 200 fixed while comparing `build_shards=1` with the `build_shards=4`
per-partition BFS-prefix union. Each arm includes persisted-head and
owner-oracle variants. The existing 100k registration is
`../002-union-construction/artifacts/task207-100k-union-ab.json`; its completed
run will be captured here if required for closeout.

The 10k diagnostic already recorded a candidate recall increase from 0.9529 to
0.9615 and a physical storage increase from 242,745,344 to 244,285,440 bytes.
The 50k and 100k results are intentionally left open until the suite artifacts
land.

Please review the preregistration and leave findings under this packet's
`feedback/` directory.
