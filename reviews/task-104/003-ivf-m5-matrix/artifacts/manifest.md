# Task 104 packet 003 — IVF M5 matrix: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `003-ivf-m5-matrix/`
- Branch: `task-104-m5-bench-optimization`
- Host: Apple M5 Pro, `aarch64-apple-darwin`, PostgreSQL 18.3 (Homebrew
  binaries, pgrx-managed cluster `~/.pgrx/data-18`, socket
  `/Users/peter/.pgrx`, port 28818)
- Backend: release (recorded as `backend.build_profile=release` in this packet's `suite-manifest.json`), dylib sha256
  `11cc8654b91bf920dc1f9d07a9523b34ff4f36c39ed76f184663c43b925c80fe`
  (install log `002-hnsw-m5-matrix/artifacts/install-ecaz-pg18.log`),
  built from head `16133580a`.
- Isolation: one index per table, `task104_*` prefixes, isolated bench
  database `tqvector_bench`.
- Runner: `ecaz bench suite run --config <packet SuiteConfig>
  --continue-on-error --manifest-output artifacts/suite-manifest.json`
  (exact expanded commands recorded per step in `suite-manifest.json`).
- Date: 2026-06-11.

- SuiteConfigs: `task104-ivf-m5-matrix-suite.json` (22 steps) plus
  `task104-ivf-batch-cells-suite.json` (12 corrected cells - the initial
  cells omitted `--ivf-scratch-soa-batch-decode`, the IVF batch axis used
  by Task 93; without it no IVF cell engages the batch surface).
- NOTE: `results.jsonl` was overwritten by the rb4-only re-run (the
  re-run omitted `--results-output`); it now contains only the rb4
  cells. The full-run evidence remains in the per-cell `recall-*.log` /
  `latency-*.log` files and `suite-run.log`; the corrected batch cells
  are in `results-batch-cells.jsonl` + `suite-manifest-batch-cells.json`.
- Key results:
  - TQ no-QJL batch-on: 221.3-221.9 ns/c at isa=neon, scalar_candidates=0,
    width histogram block-dominant (1158 ge32 vs 27 sub-32 flushes at
    nprobe=8); one-off anchor 880.9-912.9 ns/c -> ~4.1x; e2e p50
    -53.5%/-59.3%/-62.7% (nprobe 8/16/32).
  - grouped-PQ batch-on: 30.4-30.9 ns/c at isa=neon over 980k candidates.
  - RaBitQ bits=1 batch-on: 63.9-64.1 ns/c at isa=neon.
  - RaBitQ storage-bits sweep at nprobe=16: bits=2 recall 0.9410/p50
    3.07ms; bits=4 recall 0.9750; bits=8 recall 0.9820/p50 0.53ms.
  - rerank=off and adaptive-nprobe option cells recorded in per-cell logs.
  - Recall: all batch-on/off pairs byte-equal.
