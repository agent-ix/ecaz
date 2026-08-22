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
- Runtime status: pending exact-head release rebuild/install and fresh
  synthetic suite run. No concurrency, retry, saturation, recall, latency, or
  storage result is claimed yet.
