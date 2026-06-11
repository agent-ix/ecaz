# Task 104 packet 002 — HNSW M5 matrix: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `002-hnsw-m5-matrix/`
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

- SuiteConfig: `task104-hnsw-m5-matrix-suite.json` (26 steps; 24 completed,
  2 recorded-skipped `structurally_absent` markers for the HNSW rabitq
  bits=4/8 storage lanes).
- Key results (`results.jsonl`, per-cell logs):
  - full_lut kernel-on: 459.6-520.4 ns/c at isa=neon, scalar_candidates=0;
    same-cell one-off anchor 900-913 ns/c -> floor ratio ~1.83x.
  - int8_approx kernel-on: 98.7-105.0 ns/c at isa=neon (fastest HNSW
    exact mode on M5); e2e p50 -10.4%/-13.8%/-12.5% (ef 32/80/200).
  - tiled_lut retired-confirmation cell: 100% scalar (1339.6 ns/c) - the
    NEON tiled_lut path is a scalar-delegating stub; `kernel_status=retired`
    marker row emitted.
  - rabitq lane: ~65 ns/c at neon; e2e -6.5%/-10.3%/-5.7%.
  - grouped-PQ: zero batch counters with binary prefilter disabled -
    coverage gap recorded for the Task 94/101 lane (not fixed here).
  - Recall: every kernel-on/off pair byte-equal (see packet 008).
