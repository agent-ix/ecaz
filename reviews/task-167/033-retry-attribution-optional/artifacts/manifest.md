# Task 167 packet 033 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `033-retry-attribution-optional`.
- Product/harness checkpoint: `c9c9628eb`.
- Trigger: packet 032 synthetic diagnostic 1 failed at exact head
  `cdecb75e4` because `ec_distann_retry_attribution` was absent during the
  first serving query.
- Code behavior: retry attribution is optional when the fixture relation is
  absent; the fixture uses `public.ec_distann_retry_attribution` consistently.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo check --no-default-features --features pg18`.
- Validation result: passed in `validation-check.log`.
- Exact runtime head: `1c70b57d29c5c1cd382cc086fb157145be06ae67`.
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build --release -p ecaz-cli`.
  Log: `build-cli.log`; SHA-256:
  `b80a2bf8aac297a0aa96d63004aab739131ceebc68e2b00eb313926f8e7d2f00`.
- PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
  Log: `install-extension.log`; SHA-256:
  `d6c3d338f8709af0668b18d1a59ecf460abc05213aab4028f69fa876dfa369d8`.
- Installed artifact provenance: CLI SHA-256
  `6f82896a295f9f0214126cc8c321679272d7ffe4b98827daddd90ae7ea0517ef`;
  PG18 `ecaz.so` SHA-256
  `7f6df3cc6190772bd03d97c207e883c8257c7c075a76e5040e5faa8a4cf98ef1`.
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/033-retry-attribution-optional/artifacts/audit --audit`.
  Result: passed in `suite-audit.log`.
- Synthetic command:
  `/home/peter/.cargo-target/release/ecaz bench suite --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/033-retry-attribution-optional/artifacts/smoke-synthetic-remediated --only concurrency-synthetic`.
- Lane / fixture / storage / rerank: synthetic, 2,000 rows, dimension 4,
  three physical owners, graph degree 8, head cap 4,096, beam width 4,
  20 hop rounds; physical generation storage; no rerank variant.
- Cluster isolation: one multinode fixture and one logical index for this
  synthetic gate. Run directory was
  `/home/peter/.ecaz/clusters/task167-recovery-20260821-synthetic`, outside the
  repository as required. The cluster is operational state, not evidence.
- Runtime result: release preflight passed at exact head `1c70b57d2` with
  `extension_build_profile=release`, `extension_features=pg18`, and
  `debug_override=false`; ready/published topology passed for all three owners;
  `physical_serving pass=true rows=10 owners=3 source_rows=2000`.
- Failure boundary: the later full-owner materialization query aborted the
  coordinator backend with PostgreSQL assertion
  `TransactionIdFollowsOrEquals(xid, TransactionXmin)` in `subtrans.c:169`.
  The stack enters `generation_read::lookup_graph_nodes`; suite status is
  failed with exit code 1. Evidence:
  `smoke-synthetic-remediated/concurrency-synthetic/node1-postgres.log`,
  `smoke-synthetic-remediated/concurrency-synthetic/distann-local-multinode.log`,
  and `smoke-synthetic-remediated/suite-manifest.json` (SHA-256
  `2ad53466e57b16e24037a1119bc50294133fd218b7a8486b3f0efa7018c6fdf2`).
- The 10k/50k/100k steps were not selected. No concurrency, natural-retry,
  saturation, recall, latency, or storage result is claimed by this packet.
- Cleanup note: after all three `pg_ctl status` checks reported no running
  server, removing the exact failed run directory caused the old node-1
  postmaster (which had been recovering after its assertion abort without a
  usable lock file) to append an immediate-shutdown sequence to
  `node1-postgres.log`. No new query evidence was produced; the durable log now
  includes that terminal shutdown boundary.
