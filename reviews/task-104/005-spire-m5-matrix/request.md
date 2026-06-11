# Task 104 packet 005 — SPIRE M5 matrix (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 2
- Branch: `task-104-m5-bench-optimization`; measured at head `16133580a`
- No code change under review; measurement evidence packet.
- Evidence: `task104-spire-m5-matrix-suite.json`, `artifacts/manifest.md`,
  `artifacts/suite-manifest.json`, `artifacts/results.jsonl`, per-cell logs.

SPIRE column: TQ no-QJL batch on/off (226.8 ns/c, 3.61x floor, e2e -9.6
to -12.4%), rabitq lane, leaf-block option cell, nprobe {8,16,32}.
**Finding for Task 99**: SPIRE PqFastScan is structurally absent
end-to-end — the reloption parses but assignment encoding unconditionally
errors ("requires a persisted grouped-PQ model"); no end-to-end SPIRE PQ
evidence exists on any host. Marked `structurally_absent` in the matrix.
