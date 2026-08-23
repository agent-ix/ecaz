# Task 167 packet 047 artifact manifest — preregistration

- Preregistration head: `63a78b6f0`.
- Owning packet: `reviews/task-167/047-shipped-only-50k/`.
- Suite config: `task167-shipped-only-50k-suite.json`.
- Suite config SHA-256:
  `eb24ee89c998215b191384741b5f0bfa86734731a6d56ef8ab336dc616367cdb`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-shipped-only-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 shipped-default robust-prune
  inserts with ID base `2000000`. The append-when-room diagnostic ID base
  `3000000` is excluded until after the quality gate passes.
- Hard gates embedded by code checkpoint `c3b01290b`: inserted-neighborhood
  deficit at most `0.015`, heldout deficit at most `0.007`; source packet 045.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed.

## Commands

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/047-shipped-only-50k/artifacts/task167-shipped-only-50k-suite.json --log-file reviews/task-167/047-shipped-only-50k/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `ef40d406606a7ba2dcedaa8235758781959e4871d625eba3e4bfb8cd6e9a7a78`.
- Run after exact-runtime build and audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/047-shipped-only-50k/artifacts/task167-shipped-only-50k-suite.json --log-file reviews/task-167/047-shipped-only-50k/artifacts/suite-run.log`.
