# Task 104 packet 002 — HNSW M5 matrix (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 2
- Branch: `task-104-m5-bench-optimization`; measured at head `16133580a`
- No code change under review; measurement evidence packet.
- Evidence: `task104-hnsw-m5-matrix-suite.json`, `artifacts/manifest.md`,
  `artifacts/suite-manifest.json`, `artifacts/results.jsonl`, per-cell logs.

HNSW column of the M5 matrix: TQ exact-mode sweep (full_lut / int8_approx /
tiled_lut-retired / exact), grouped-PQ, rabitq sidecar lane, batch on/off,
ef_search {32,80,200}. Headlines: full_lut (lut32 NEON repack, the Task 104
"first suspect") passes the 1.5x floor at ~1.83x — no revert-to-v1;
int8_approx at 99-105 ns/c is the fastest HNSW exact mode on M5; tiled_lut
confirmed retired (NEON path is a scalar stub); recall byte-equal on every
on/off pair. HNSW grouped-PQ batch non-engagement recorded as a Task 94/101
coverage gap, not fixed here. The HNSW rabitq bits=4/8 lanes are
structurally absent (no `quant_bits` on ec_hnsw) and recorded as skipped
marker rows.
