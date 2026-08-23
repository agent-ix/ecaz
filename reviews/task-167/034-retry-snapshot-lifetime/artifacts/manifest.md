# Task 167 packet 034 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `034-retry-snapshot-lifetime`.
- Code checkpoint: `15f7fcf5f`.
- Trigger evidence: packet 033
  `artifacts/smoke-synthetic-remediated/concurrency-synthetic/node1-postgres.log`.
- Failure signature: PostgreSQL assertion
  `TransactionIdFollowsOrEquals(xid, TransactionXmin)` in `subtrans.c:169`;
  symbolicated extension frame `generation_read::lookup_graph_nodes`.
- Root cause: `GenerationExpander` preserved a refreshed snapshot's raw
  pointer after its `RegisteredSnapshotGuard` dropped at the end of the prior
  expansion call.
- Remediation: initial successful lookups retain the caller snapshot;
  refreshed retry snapshots remain registered in the expander for later
  traversal rounds.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo check --no-default-features --features pg18`.
- Validation result: passed in `validation-check.log` (SHA-256
  `ce62fc06053c282beeb0e1230d67fa57f98553f7667b5c9f11bef3021584e083`).
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build --release -p ecaz-cli`.
  Log: `build-cli.log`; SHA-256
  `ed53e6e198333765fff3fcab4d3f97c28fc33f96d8ade4900a8368734bc9a4be`.
- PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
  Log: `install-extension.log`; SHA-256
  `a5a5d65a81164d84d00a7c028c03202f7788052bf498a878bc923819c90ad3ef`.
- Built artifact hashes: CLI
  `6ee15ecb7aa8fac8327159e24d45c65258e1ac0d6f6e11898e4dd205ab915aa0`;
  installed PG18 `ecaz.so`
  `c14052c62b4e803dd97968226ceea346b7e43717e4680d5f53a260aa325a6dba`.
- Provenance limitation: both artifacts embedded
  `b33af0342f62a12c15e8de5574c94756f93a7a2c-dirty`, because this packet's
  generated logs existed before Cargo captured the Git state. The run is
  diagnostic and is not claimed as exact-clean-head evidence.
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --log-file reviews/task-167/034-retry-snapshot-lifetime/artifacts/suite-audit.log`.
  Result: passed, four steps.
- Synthetic command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/034-retry-snapshot-lifetime/artifacts/smoke-synthetic-remediated --only concurrency-synthetic --log-file reviews/task-167/034-retry-snapshot-lifetime/artifacts/smoke-synthetic-remediated/suite-run.log`.
- Lane / fixture / storage / rerank: synthetic, 2,000 rows, dimension 4,
  three physical owners, graph degree 8, head cap 4,096, beam width 4,
  20 hop rounds; physical generation storage; no rerank variant. One logical
  index was isolated in the external run directory
  `/home/peter/.ecaz/clusters/task167-recovery-20260821-synthetic`.
- Snapshot-fix result: release preflight and ready/published topology passed;
  `physical_serving pass=true`; remote-owner node 2 and node 3 both reported
  `custom_scan=true pass=true`. No PostgreSQL assertion or backend crash
  occurred.
- Later failure: `physical_mid_insert_failure pass=true`, but the concurrent
  drill reported controlled target neighbor count `6 -> 6`, natural retries
  `0`, steady retries `0`, reverse-edge coverage `10/24`, and
  `physical_concurrent_insert_query pass=false`. Suite exit code was 1.
- Evidence:
  `smoke-synthetic-remediated/concurrency-synthetic/distann-local-multinode.log`,
  owner PostgreSQL logs, and
  `smoke-synthetic-remediated/suite-manifest.json`.
- The 10k/50k/100k steps were not selected. No accepted concurrency, retry,
  saturation, recall, latency, or storage result is claimed.
