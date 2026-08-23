# Task 167 packet 054 artifact manifest — measured negative

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
  are not committed. The three generated `nodeN-postgres.log` files were
  removed before checkpointing, and the stopped 5 GB external fixture was
  removed after the cited result artifacts were captured.
- Run timestamp: `2026-08-22T21:15:22-07:00`; finish timestamp:
  `2026-08-22T21:51:22-07:00`; duration `2,163,921 ms` (`36m03.921s`).
  Suite status: failed at the preregistered heldout quality gate.

## Preregistered command

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/054-conservative-admission-50k-gate/artifacts/task167-conservative-admission-50k-suite.json --log-file reviews/task-167/054-conservative-admission-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `52bc0d15827f5eee1ab8f6a8f44071f58ab9cafa192f5b63a231adb2cef55243`.
- Run, only after exact-runtime PG18 release installation, release CLI build,
  and repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/054-conservative-admission-50k-gate/artifacts/task167-conservative-admission-50k-suite.json --log-file reviews/task-167/054-conservative-admission-50k-gate/artifacts/suite-run.log`.
- Report after the expected failed-step exit:
  `/home/peter/.cargo-target/release/ecaz bench suite report --manifest reviews/task-167/054-conservative-admission-50k-gate/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/054-conservative-admission-50k-gate/artifacts/final-suite/results.jsonl --log-file reviews/task-167/054-conservative-admission-50k-gate/artifacts/suite-report.log`.

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

## Result and disposition

- Pre-insert recall instrumentation reconciled exactly: ordinary and exact
  distinct recall were both `0.954500` (`absolute_delta=0.000000`).
- Insert throughput: physical `0.227` rows/s versus single-instance control
  `0.420` rows/s (`physical_over_control=0.539958`). This unjudged measurement
  is not used to rescue or reject the candidate.
- Conservative admission rejected 133 backlink amendments out of 4,987
  attempted amendments; `backlink_prune_rejected=133`.
- Inserted-neighborhood exact recall: physical `0.923735`, fresh `0.931052`,
  deficit `0.007316`; fixed allowance `0.015000`; pass.
- Heldout exact recall: physical `0.847722`, fresh `0.857333`, deficit
  `0.009611`; fixed allowance `0.007000`; fail by `0.002611`.
- The unconditional-append arm was skipped with
  `control_mutation_excluded=true`, and post-gate drills were skipped.
- Against the same fixed query/truth surface, the candidate's heldout deficit
  is `0.001000` better than packet 051 append-only (`0.010611`) and `0.001000`
  worse than packet 047 robust-prune-all (`0.008611`). The candidate is
  rejected; no final 10k/50k/100k closeout matrix is authorized.
- The cited result lines are copied verbatim into `cited-results.log`.

## Committed result artifacts

- `suite-run.log`: SHA-256
  `d2baf81fd5fc90cbac65b552eb5558f6203bd895d57217ec7ae9bac438cb83d4`.
- `suite-report.log`: SHA-256
  `247599aec8a63c8f172496519259cad2f64a81032187473baa567e4cd5803a50`.
- `final-suite/suite-manifest.json`: SHA-256
  `61627fc1e81a670127983b6ac15caca02319312102ee9a5c557c75d607769b39`.
- `final-suite/results.jsonl`: 88 structured rows; SHA-256
  `051ccb498603f0ea38d6903597260618484404ebae0b0d8fe62efcac5d21b11c`.
- `distann-local-multinode.log`: SHA-256
  `4f02d2c5af0f72297af973c2d8e85711f7eed4ddc66a21fa965984448c9aeec6`.
- `distann-multinode-summary.log`: SHA-256
  `2cb1e13445fc849d24850f1056fd473ac6a7ce0e73da115f0ee13ffad3499f71`.
- `physical-production-recall.log`: SHA-256
  `3300a4047432574297553ecb2fa815e94f65b2a359bbc95307c432f6f22503fc`.
- `physical-production-latency.log`: SHA-256
  `60e37bca166fbf1f571ba2302d97ca6ddb95cd4624b8a1332ba4f36bfcfde825`.
- `physical-production-predictions.json`: SHA-256
  `1abc27ffe21b97f0513c35721e4e354fd2b379dbd7d7a7031fc009ac5f219e22`.
- `physical-head-membership.json`: SHA-256
  `c76c218365faec135ad1a4e00009d3f4058d1fb8644614efb81159f138ccad45`.
