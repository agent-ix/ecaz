# Task 167 packet 054 artifact manifest — preregistration

- Preregistration head: `9c57ff59a6b9d54326a9abc7d17fc8353a8a43d4`.
- Candidate code: `4826e96447911d33e915943f591eebdf6a80ce06`.
- Owning packet: `reviews/task-167/054-conservative-admission-50k-gate/`.
- Suite config: `task167-conservative-admission-50k-suite.json`.
- Suite config SHA-256:
  `149019a894ea2bc3cc8684dde205b5f356bebeb51bd4c7d7aca1c213864452e3`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-conservative-admission-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 shipped-default conservative
  admission inserts with ID base `2000000`. The unconditional-append
  diagnostic ID base `3000000` is excluded until after the quality gate passes.
- Hard gates remain fixed from packet 045: inserted-neighborhood deficit at
  most `0.015`, heldout deficit at most `0.007`.
- Before comparators:
  - packet 047 robust-prune-all heldout deficit `0.008611`;
  - packet 051 unconditional append heldout deficit `0.010611`.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed.

## Preregistered command

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/054-conservative-admission-50k-gate/artifacts/task167-conservative-admission-50k-suite.json --log-file reviews/task-167/054-conservative-admission-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `52bc0d15827f5eee1ab8f6a8f44071f58ab9cafa192f5b63a231adb2cef55243`.
- Run, only after exact-runtime PG18 release installation, release CLI build,
  and repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/054-conservative-admission-50k-gate/artifacts/task167-conservative-admission-50k-suite.json --log-file reviews/task-167/054-conservative-admission-50k-gate/artifacts/suite-run.log`.

## Exact runtime

- Runtime head: `80107d843e9f55d185f102455227be7d09648709`;
  both the installed extension and release CLI embed this SHA with profile
  `release`.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. `install-extension.log` LF-normalized SHA-256:
  `2e7f5d42831aab77f25db7f0ddf0f146039d1db126775aa2d6a752f98a375369`.
- Installed `ecaz.so` SHA-256:
  `d808fd6a2809e1a867cbe50bbdb965f5c6ab2b40875b3a40e4ae3f8badb7fd82`.
- CLI build command: `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed with the pre-existing unrelated dead-code warning at
  `commands/corpus/load.rs:190`. `build-cli.log` LF-normalized SHA-256:
  `e22d2c41ae5b5a606d7b878cca69d107d9330ecbe9531203a8b5f99a78526c30`.
- Release CLI SHA-256:
  `17a1dbe23716a5eda3cb2094004b93bd82649bc4cb00384b0989ed6293f1020a`.
- Exact-runtime audit result: passed, 1 step. Log SHA-256:
  `52bc0d15827f5eee1ab8f6a8f44071f58ab9cafa192f5b63a231adb2cef55243`.

## Decision rule

- If either fixed quality band fails, reject the candidate, skip the
  unconditional-append control and post-gate drills, and keep Task 167 open.
- If both fixed quality bands pass, proceed to the packet's remaining drills
  and then preregister the isolated 10k/50k/100k recall, latency, and storage
  matrix. A 50k pass alone is not closeout.
