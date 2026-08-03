---
task: 207
packet: 001-construction-contract
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 001
---

# Review request: partition-union construction contract

Head SHA: `3fb1319af4dc0a1ebe1dc2c94138ad767fb05593`

This checkpoint reconciles the FR-080 Task 207 construction contract:

- each shard records its local Vamana medoid and a bounded BFS prefix;
- the physical head is built from the deterministic, deduplicated union of
  those per-partition prefixes rather than the stitched global graph;
- the union path is used for both the physical workspace and the legacy flush
  path, while training-policy selection remains an explicit override;
- `build_shards` is exposed through the multinode fixture and suite so the
  construction can be measured without changing the production default;
- the coordinator head cache is bounded to a two-entry LRU with logical index,
  build, and epoch-derived identity plus metadata fingerprint validation.

The persisted Vamana head-search path remains the existing search path; this
packet does not claim the full-scale measurement gate. Validation is recorded
in `artifacts/validation.md`, and the required 10k/50k/100k A/B evidence is
still open because this host has no staged corpus files.

Please review the code checkpoint and leave findings under this packet's
`feedback/` directory.
