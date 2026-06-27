# Task 104 packet 006 — QJL 1024-dim matrix, pre-fix baseline (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope items 2-3
- Branch: `task-104-m5-bench-optimization`; measured at head `16133580a`
- No code change under review; measurement evidence packet.
- Evidence: `task104-qjl-1024dim-m5-matrix-suite.json`,
  `artifacts/manifest.md`, suite manifest, `results.jsonl`, fixture TSVs,
  per-cell logs.

The non-1536-dim fixture required by the task (synthetic isotropic 10k @
1024-dim) exercising the QJL lanes on HNSW/IVF/SPIRE. **This packet is the
floor-gate failure that triggered the optimization arm**: qjl32 NEON block
kernel at 666.8-683.9 ns/c vs 535.5-579.8 ns/c one-off anchor (0.83x).
DiskANN TQ @1024 is structurally absent (1536-only lane) and marked.
Recall on/off byte-equal. Superseded for performance by packet 007.
