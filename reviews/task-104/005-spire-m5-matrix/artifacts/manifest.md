# Task 104 packet 005 — SPIRE M5 matrix: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `005-spire-m5-matrix/`
- Branch: `task-104-m5-bench-optimization`
- Host: Apple M5 Pro, `aarch64-apple-darwin`, PostgreSQL 18.3 (Homebrew
  binaries, pgrx-managed cluster `~/.pgrx/data-18`, socket
  `/Users/peter/.pgrx`, port 28818)
- Backend: release (`ecaz_build_profile()` probe in
  `002-hnsw-m5-matrix/artifacts/build-profile-probe.log`), dylib sha256
  `11cc8654b91bf920dc1f9d07a9523b34ff4f36c39ed76f184663c43b925c80fe`
  (install log `002-hnsw-m5-matrix/artifacts/install-ecaz-pg18.log`),
  built from head `16133580a`.
- Isolation: one index per table, `task104_*` prefixes, isolated bench
  database `tqvector_bench`.
- Runner: `ecaz bench suite run --config <packet SuiteConfig>
  --continue-on-error --manifest-output artifacts/suite-manifest.json`
  (exact expanded commands recorded per step in `suite-manifest.json`).
- Date: 2026-06-11.

- SuiteConfig: `task104-spire-m5-matrix-suite.json` (10 steps; 9 completed,
  1 recorded-skipped `structurally_absent` marker for SPIRE PqFastScan).
- Key results:
  - TQ no-QJL kernel-on: 226.8 ns/c avg at isa=neon vs 818.8 ns/c one-off
    -> 3.61x; e2e p50 -9.6%/-12.4%/-11.9% (nprobe 8/16/32).
  - SPIRE PqFastScan is structurally absent end-to-end: the reloption
    parses but `encode_assignment_payload` unconditionally errors
    ("requires a persisted grouped-PQ model"); no fixture flow can build
    the index on any host. Flagged to Task 99 as a product gap.
  - leaf_block_rows=256 option cell recorded.
  - RaBitQ lane measured e2e (counters not batch-attributed on this
    surface).
  - Recall: all kernel-on/off pairs byte-equal.
