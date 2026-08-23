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

## Exact runtime

- Runtime head: `8bf0ac8a451f9cd73813dd0ab59ed305fab026bd`;
  both installed extension and release CLI embed this SHA with profile
  `release`.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. Committed `install-extension.log` SHA-256:
  `580f1405be92924ea6a7275b623e98aa3ffe8ed5bce8b2971066cef9bc03a4c1`.
- Installed `ecaz.so` SHA-256:
  `49f21b5151d071ba9709d6ba2a4f11c72653011361ed6b48a6ab2bedb6f6bd59`.
- CLI build command: `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed with the pre-existing unrelated dead-code warning at
  `commands/corpus/load.rs:190`. Committed `build-cli.log` SHA-256:
  `eef89e71f15898d25765f73559ad9d9906144200bbe7cce700d88b36ac1c5760`.
- Release CLI SHA-256:
  `3f96c106338597793049b067bb3687bee955e4cdc6f691e8e2615e306077353a`.
- Exact-runtime audit result: passed, 1 step. Log SHA-256:
  `ef40d406606a7ba2dcedaa8235758781959e4871d625eba3e4bfb8cd6e9a7a78`.
