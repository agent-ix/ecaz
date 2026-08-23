# Task 167 packet 051 artifact manifest — preregistration

- Preregistration head: `3736324235d918e1c3fb622881cf38e7919f3e0b`.
- Candidate code: `22c1e01c3f7dfb188f3f38c2022b5208252825e4`.
- Owning packet: `reviews/task-167/051-batch-consistent-50k-gate/`.
- Suite config: `task167-batch-consistent-50k-suite.json`.
- Suite config SHA-256:
  `8a424a97954b6beafd9f000878d7767db21b05e6dd13ea6ddaa9412251c2d1df`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-batch-consistent-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 shipped-default
  append-when-room inserts with ID base `2000000`. The robust-prune-all
  diagnostic ID base `3000000` is excluded until after the quality gate passes.
- Hard gates remain fixed from packet 045: inserted-neighborhood deficit at
  most `0.015`, heldout deficit at most `0.007`.
- Before comparator: packet 047's isolated robust-prune-all arm, heldout
  physical `0.848722`, fresh `0.857333`, deficit `0.008611`.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed.

## Commands

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/051-batch-consistent-50k-gate/artifacts/task167-batch-consistent-50k-suite.json --log-file reviews/task-167/051-batch-consistent-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `d939548ad01858e6fd71102f88034830ac44930bec282d067fc95d7607239e7f`.
- Run after an exact-runtime PG18 release install, release CLI build, and
  repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/051-batch-consistent-50k-gate/artifacts/task167-batch-consistent-50k-suite.json --log-file reviews/task-167/051-batch-consistent-50k-gate/artifacts/suite-run.log`.

## Exact runtime

- Runtime head: `383423fa5edd71ef5fd8d317823032da712a173d`;
  both installed extension and release CLI embed this SHA with profile
  `release`.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. `install-extension.log` committed LF-normalized
  SHA-256:
  `289019fbe76f0b234a014cc571cc31152e38ff301c4088ce213d3c6401f278a8`.
- Installed `ecaz.so` SHA-256:
  `6d6900c25be5d25916be382d1b16afbfb29448e9f2548170cae3bc3066b72385`.
- CLI build command: `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed with the pre-existing unrelated dead-code warning at
  `commands/corpus/load.rs:190`. `build-cli.log` committed LF-normalized
  SHA-256:
  `64fcbfd5f805bb489000b3367e6e7dac1e015ce846a3d57b28f7996412afd206`.
- Release CLI SHA-256:
  `893d29d782d8699b5340e7bb65940a120c3d3d8b27c856d2c98b3a68eb451174`.
- Exact-runtime audit result: passed, 1 step. Log SHA-256:
  `d939548ad01858e6fd71102f88034830ac44930bec282d067fc95d7607239e7f`.
