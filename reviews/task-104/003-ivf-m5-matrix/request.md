# Task 104 packet 003 — IVF M5 matrix (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 2
- Branch: `task-104-m5-bench-optimization`; measured at head `16133580a`
- No code change under review; measurement evidence packet.
- Evidence: `task104-ivf-m5-matrix-suite.json`,
  `task104-ivf-batch-cells-suite.json`, `artifacts/manifest.md`,
  suite manifests, `results-batch-cells.jsonl`, per-cell logs.

IVF column: TQ no-QJL, grouped-PQ, rabitq quant_bits {1,2,4,8}, rerank
on/off, adaptive-nprobe, nprobe {8,16,32}. The initial cells missed the
IVF batch axis (`--ivf-scratch-soa-batch-decode`, per the Task 93
precedent); the corrected batch cells show TQ at 221 ns/c (4.1x floor,
e2e p50 up to -62.7%), grouped-PQ at 30.4-30.9 ns/c, rabitq at 64 ns/c,
all at isa=neon / scalar_candidates=0 with block-dominant width
histograms. Recall byte-equal on every pair. Note the results.jsonl
overwrite caveat in the manifest (per-cell logs are authoritative for the
initial run).
