# Task 104 packet 007 — qjl32 NEON candidate-parallel kernel + post-fix evidence: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `007-qjl32-neon-optimization/`
- Branch: `task-104-m5-bench-optimization`
- Host: Apple M5 Pro, `aarch64-apple-darwin`, PostgreSQL 18.3 (Homebrew
  binaries, pgrx-managed cluster `~/.pgrx/data-18`, socket
  `/Users/peter/.pgrx`, port 28818)
- Backend: release (`ecaz_build_profile()` probe in
  `002-hnsw-m5-matrix/artifacts/build-profile-probe.log`), dylib sha256
  `see body`
  (install log `002-hnsw-m5-matrix/artifacts/install-ecaz-pg18.log`),
  built from head `16133580a`.
- Isolation: one index per table, `task104_*` prefixes, isolated bench
  database `tqvector_bench`.
- Runner: `ecaz bench suite run --config <packet SuiteConfig>
  --continue-on-error --manifest-output artifacts/suite-manifest.json`
  (exact expanded commands recorded per step in `suite-manifest.json`).
- Date: 2026-06-11.

- Code under review: `f88c640d3` (candidate-parallel qjl32 NEON block
  kernel, aarch64-only; see request.md).
- Backend: fresh release install, dylib sha256
  `a11db8fb54a54b7f28b608e3148cd21211e44915f7bc333b7b92dad7ee826e73`
  (`install-ecaz-pg18.log`), postmaster restarted, probe
  `build-profile-probe.log` = release.
- SuiteConfigs: `task104-qjl32-neon-postfix-suite.json` (10 cells, re-runs
  the packet 006 bench cells on the same fixtures/indexes) and
  `task104-ivf-qjl-batch-cells-suite.json` (4 corrected IVF QJL cells with
  the batch axis).
- Key results:
  - qjl32 NEON kernel: 167.3-184.5 ns/c at isa=neon (was 666.8-683.9) -
    ~4x kernel speedup; floor ratio 0.83x -> 3.2-3.5x vs the 581.9-589.7
    ns/c one-off anchor.
  - IVF QJL e2e p50: -46.3%/-53.8% (nprobe 8/16) batch-on vs off.
  - SPIRE QJL e2e p50: -3.6%/-7.5%.
  - HNSW QJL e2e neutral (+1.5/+2.0%) - exact path is one-off dominated
    (~3k batch vs ~138-225k one-off candidates); recorded as-is.
  - Recall identical to packet 006 pre-fix values at every cell
    (tolerance lane holds; 4-ulp unit gates in packet 001 cover the
    kernel contract).
