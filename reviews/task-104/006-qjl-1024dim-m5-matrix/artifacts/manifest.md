# Task 104 packet 006 — QJL 1024-dim M5 matrix (pre-fix): artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `006-qjl-1024dim-m5-matrix/`
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

- SuiteConfig: `task104-qjl-1024dim-m5-matrix-suite.json` (16 steps; 15
  completed, 1 `structurally_absent` marker for DiskANN TQ @1024 -
  ambuild requires the no-QJL 4-bit 1536 lane).
- Fixtures: synthetic isotropic 10k corpus / 64 queries at dim=1024
  (seeds 10401/10402, generated in-suite; TSVs in artifacts/).
- Key result - **floor-gate FAILURE that triggered the Task 104
  optimization arm**: qjl32 NEON block kernel 666.8-683.9 ns/c vs
  535.5-579.8 ns/c one-off anchor (ratio 0.83x) - batching was a
  pessimization on Apple silicon. Recall on/off byte-equal at every cell.
- Superseded for performance by packet 007 (kernel rewrite); kept as the
  pre-fix baseline.
