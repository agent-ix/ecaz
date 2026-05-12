---
topic: spire-remote-pk-read-isolation
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30937
stage: phase-12.6
status: open
---

# Review Request: SPIRE Remote PK Read Isolation

## Scope

Please review commit `ada38951be9152c3e7fcb271954a43679648d3e4`
(`Pin SPIRE remote PK read isolation contract`).

This slice closes the Phase 12.6 EvalPlanQual/isolation decision item for the
remote PK-read path:

- Adds `test_ec_spire_remote_pk_select_isolation_contract_sql`.
- The fixture creates a loopback coordinator table plus remote shard table,
  registers a remote descriptor, and confirms `SELECT ... WHERE id = 2606`
  plans as `Custom Scan (EcSpireDistributedScan)`.
- It starts external coordinator transactions at `READ COMMITTED`,
  `REPEATABLE READ`, and `SERIALIZABLE`.
- In each transaction it reads the remote-owned row, commits a concurrent
  remote update from another backend, reads the same PK again, and asserts the
  second read observes the newer remote title.
- It leaves `ec_spire_custom_scan_recheck` unconditional for v1 and adds the
  code comment tying that choice to virtual remote tuple payloads.
- It documents the accepted v1 isolation contract in ADR-068 and ADR-069:
  remote CustomScan payloads do not carry coordinator heap row identity,
  remote statements do not inherit the coordinator transaction snapshot, and
  EvalPlanQual cannot rerun against the remote origin row.
- It marks the Phase 12.6 isolation/recheck tracker rows complete.

## Review Focus

- Confirm the fixture is exercising the real remote PK-read CustomScan path,
  not a local-node stand-in.
- Confirm the expected v1 behavior is acceptable: `REPEATABLE READ` and
  `SERIALIZABLE` coordinator transactions can observe a newer remote row on a
  later remote dispatch in the same transaction.
- Confirm ADR-068/ADR-069 state the limitation clearly without overstating
  broader distributed-query guarantees.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql`

Key result: `1 passed; 0 failed; 1688 filtered out`.
