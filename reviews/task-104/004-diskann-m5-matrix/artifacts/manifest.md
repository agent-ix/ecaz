# Task 104 packet 004 — DiskANN M5 matrix: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `004-diskann-m5-matrix/`
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

- SuiteConfig: `task104-diskann-m5-matrix-suite.json` (18 steps, all
  completed).
- Key results:
  - TQ no-QJL kernel-on: 298.7 ns/c avg at isa=neon vs 891.1 ns/c one-off
    -> 2.98x; e2e p50 -35.2%/-38.3% (list_size 64/128).
  - binary/Hamming sidecar: 7.1 ns/c at isa=neon (integer-exact contract,
    Task 95 family); sidecar on/off cells recorded.
  - RaBitQ bits=1: 65.6 ns/c at isa=neon.
  - grouped-PQ storage cells route scoring through the binary sidecar
    prefilter under prefilter_kind=auto (`quant=binary` counters) -
    recorded as observed routing, grouped-PQ batch arm engages only via
    prefilter_kind=grouped_pq.
  - Recall: all kernel-on/off pairs byte-equal.
