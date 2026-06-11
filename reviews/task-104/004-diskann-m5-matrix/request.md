# Task 104 packet 004 — DiskANN M5 matrix (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 2
- Branch: `task-104-m5-bench-optimization`; measured at head `16133580a`
- No code change under review; measurement evidence packet.
- Evidence: `task104-diskann-m5-matrix-suite.json`, `artifacts/manifest.md`,
  `artifacts/suite-manifest.json`, `artifacts/results.jsonl`, per-cell logs.

DiskANN column: grouped-PQ storage (sidecar-routed under prefilter auto),
binary sidecar on/off, rabitq bits=1, TQ no-QJL, list_size {64,128}.
Headlines: TQ kernel 298.7 ns/c (2.98x floor, e2e -35/-38%); hamming
sidecar 7.1 ns/c; rabitq 65.6 ns/c; recall byte-equal everywhere.
