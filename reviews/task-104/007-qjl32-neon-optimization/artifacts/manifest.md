# Task 104 packet 007 — qjl32 NEON candidate-parallel kernel + post-fix evidence: artifact manifest

- Task bucket: `reviews/task-104/`
- Packet: `007-qjl32-neon-optimization/`
- Branch: `task-104-m5-bench-optimization`
- Host: Apple M5 Pro, `aarch64-apple-darwin`, PostgreSQL 18.3 (Homebrew
  binaries, pgrx-managed cluster `~/.pgrx/data-18`, socket
  `/Users/peter/.pgrx`, port 28818)
- Backend: release per round — dylib SHAs, install logs, and
  `backend.build_profile=release` suite-manifest records are listed in
  the body below per measurement round.
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
  (`install-ecaz-pg18.log`), postmaster restarted; release recorded as `backend.build_profile=release`
  in `suite-manifest.json` (the probe-log artifact was empty — wrong flag —
  and was dropped).
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

## Octet-round addendum (response to 2026-06-11-01-reviewer P1)

- Code: `5c44d9f45` — NEON octet entry (`score_octet8_neon`) + ISA-dispatched
  remainder routing (`score_turboquant_qjl_octet8`); the 8-31-candidate
  remainder band no longer breaks to scalar on aarch64.
- Backend: fresh release install, dylib sha256
  `fda206bea488fb9bb2cca666c1873c1e11a08ea8eb8f817ce51bf1e5f1e85bfb`
  (`install-ecaz-pg18-octet.log`), postmaster restarted, probe `build-profile-probe-octet.log` = release
  (captured via `dev sql --log-output`).
- SuiteConfig: `task104-qjl32-neon-octet-suite.json` (14 cells; manifest
  `suite-manifest-octet.json`, results `results-octet.jsonl`).
- Batch-surface scalar fallback eliminated:
  - HNSW kernel-on: kernel candidates 2,944/3,040 -> 113,488/182,952 with
    the 8-15/16-31 width buckets now on the `isa=neon` row (168.3-168.9
    ns/c); remaining scalar rows (27,878/44,662) carry empty width
    histograms — the genuine one-off (non-batch) scoring path.
  - SPIRE kernel-on: scalar 21,190/43,065 -> 5,334/10,849 (one-off only).
  - IVF batch-on: scalar 4,620/4,266 -> 1,156/978 (one-offs plus 9-10
    sub-8 flushes; sub-8 stays scalar by the width-cascade design).
- e2e p50 batch-on vs off: HNSW 1.09/2.18 vs 1.32/2.52 ms
  (-17.4%/-13.5%, previously neutral); IVF 0.35/0.53 vs 0.67/1.18 ms
  (-47.8%/-55.1%); SPIRE 3.72/6.58 vs 3.94/7.26 ms (-5.6%/-9.4%).
- Recall identical to the pre-fix and postfix rounds at every cell.
