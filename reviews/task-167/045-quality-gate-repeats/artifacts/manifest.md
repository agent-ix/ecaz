# Task 167 packet 045 artifacts — preregistration

- Preregistration head before this packet commit: `fe6580a52`.
- Task bucket and packet:
  `reviews/task-167/045-quality-gate-repeats/`.
- Suite config: `task167-quality-gate-calibration-suite.json`.
- Preregistered at `2026-08-22T14:45:19-07:00`.
- Config SHA-256:
  `f67677b20ecae74b378381c932e3e10b47c3783f4f49b957b2c4a5c51814dc22`.
- Preregistration audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/045-quality-gate-repeats/artifacts/task167-quality-gate-calibration-suite.json --log-file reviews/task-167/045-quality-gate-repeats/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 6 steps. Log SHA-256:
  `cfb28f3b554838389d6cdac0ce6eefe72dc222acdca2d80bc10a7906d183c422`.
- Command after exact-runtime build and audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/045-quality-gate-repeats/artifacts/task167-quality-gate-calibration-suite.json --log-file reviews/task-167/045-quality-gate-repeats/artifacts/suite-run.log`.
- Lane: production physical distributed DistANN on PG18, three owners, rabitq
  neighbor storage, exact fp32 truth, no rerank variant.
- Fixtures: five `ec_real_10k` repeats and one `ec_real_50k` repeat. Every step
  uses an isolated external run directory, port range, corpus tables, physical
  index, single control index, and fresh rebuild; one index per table.
- Search regime: beam width 4, candidate heap 32, hop rounds 100. This is the
  current shipped Task 167 regime; Task 203 owns the paper-regime drift.
- Quality populations: 48 inserted-neighborhood queries plus all 200 staged
  heldout queries at each scale.
- Gate derivation and invalidation rules are fixed in `request.md` before the
  suite is run. No result or threshold has yet been observed or selected.
- Runtime output will be packet-local under `artifacts/calibration-suite/`.
  Corpus data, truth caches, cluster state, PostgreSQL operational logs, and
  polling exhaust will not be committed.
