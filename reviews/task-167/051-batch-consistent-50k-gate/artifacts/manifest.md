# Task 167 packet 051 artifact manifest — measured negative

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
- Run timestamp: `2026-08-22T19:35:41-07:00`; duration `2,136,381 ms`
  (`35m36.381s`). Suite status: failed at the preregistered quality gate.
- Runtime output is packet-local under `artifacts/final-suite/`. Corpus data,
  truth caches, PGDATA, PostgreSQL operational logs, and polling output are not
  committed. The three generated `nodeN-postgres.log` files were removed
  before checkpointing this packet.

## Commands

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/051-batch-consistent-50k-gate/artifacts/task167-batch-consistent-50k-suite.json --log-file reviews/task-167/051-batch-consistent-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `d939548ad01858e6fd71102f88034830ac44930bec282d067fc95d7607239e7f`.
- Run after an exact-runtime PG18 release install, release CLI build, and
  repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/051-batch-consistent-50k-gate/artifacts/task167-batch-consistent-50k-suite.json --log-file reviews/task-167/051-batch-consistent-50k-gate/artifacts/suite-run.log`.
- Report after the expected failed-step exit:
  `/home/peter/.cargo-target/release/ecaz bench suite report --manifest reviews/task-167/051-batch-consistent-50k-gate/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/051-batch-consistent-50k-gate/artifacts/final-suite/results.jsonl --log-file reviews/task-167/051-batch-consistent-50k-gate/artifacts/suite-report.log`.

## Result and disposition

- Pre-insert recall instrumentation reconciled exactly: ordinary and exact
  distinct recall were both `0.954000` (`absolute_delta=0.000000`).
- Inserted-neighborhood exact recall: physical `0.917452`, fresh `0.931052`,
  deficit `0.013600`; fixed allowance `0.015000`; pass.
- Heldout exact recall: physical `0.846722`, fresh `0.857333`, deficit
  `0.010611`; fixed allowance `0.007000`; fail by `0.003611`.
- The diagnostic robust-prune-all arm was skipped with
  `control_mutation_excluded=true`, and post-gate drills were skipped.
- Packet 047's isolated robust-prune-all heldout deficit was `0.008611`.
  Append-when-room is `0.002000` worse on this measurement, so candidate
  `22c1e01c3` is rejected and no final 10k/50k/100k closeout is authorized.
- The result lines cited above are copied verbatim into `cited-results.log`.

## Committed result artifacts

- `suite-run.log`: SHA-256
  `dd3f60beba60ca5923edd3b8d04aed43108823f44d60dc6368f99c7b88d47305`.
- `suite-report.log`: SHA-256
  `0f19dc4afd92653a68a1688a272dbcef0e8e41a2148c99b914efdf1a9b8c730d`.
- `final-suite/suite-manifest.json`: SHA-256
  `86cdc7f2b63e590bfe473426d9869ead0eb5cf2e0640ab623949a28113a766c2`.
- `final-suite/results.jsonl`: 86 structured rows; SHA-256
  `72fb718b2cb498c29a2bce21edc371658d62272ab7cb3f635d3c492bf94fd772`.
- `distann-local-multinode.log`: SHA-256
  `7d4328fda943904eda003fd7c7d6caebd92865052ebf272e0f756cca64756693`.
- `distann-multinode-summary.log`: SHA-256
  `baf0610e40168b333593b593ae394fe4734b40594ce113fc56c0411749645e6a`.
- `physical-production-recall.log`: SHA-256
  `2a658df877ba2d569acf8af0761f2c0b2cd6dc040e729e136135589164e12fa2`.
- `physical-production-latency.log`: SHA-256
  `dcec1192a15eda3203257f18d09467a4a6e454cc77e98a2fbd231565ee2aa048`.
- `physical-production-predictions.json`: SHA-256
  `11e36680b36cdb6c1a4f90afd9053201cf288dc77fdc80a9a682ab88e19cee68`.
- `physical-head-membership.json`: SHA-256
  `c76c218365faec135ad1a4e00009d3f4058d1fb8644614efb81159f138ccad45`.

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
