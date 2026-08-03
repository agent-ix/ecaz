---
task: 207
packet: 002-union-construction
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 002
---

# Review request: pre-registered partition-union A/B

Code head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

`artifacts/task207-100k-union-ab.json` is the canonical `ecaz bench suite`
configuration for the primary Task 207 construction A/B. It holds BW=128,
H=5, head cap 4096, top-k 200, and seed count 200 fixed while comparing:

- control: `build_shards=1`, stitched-graph head construction;
- candidate: `build_shards=4`, per-partition BFS-prefix union construction.

Each step includes persisted-head and owner-oracle variants so membership and
end-to-end movement can be separated. The two fixture run directories are
outside the repository. Audit/dry-run output is in `artifacts/suite-dry-run.md`.

The packet also contains a short 10k diagnostic A/B config at
`artifacts/task207-10k-union-ab.json`, using the same fixed BW=128/H=5 shape
as the 100k pre-registration. Its result rows will be added after the run;
the required 10k/50k/100k closeout remains open.
