# 30775 Artifact Manifest

- Head SHA: `f607c37fec14ec2c92d25006e884adf2805f5268`
- Packet: `30775-spire-boundary-replica-identity-snapshot`
- Timestamp: `2026-05-10T22:24:26Z`
- Lane: Phase 11.2 SPIRE boundary-replica global identity diagnostics
- Fixture: local PG18 pgrx fixture
- Storage format: SPIRE active epoch leaf assignments, `source_identity = 'include'`
- Rerank mode: not applicable
- Surface style: isolated one-index-per-table fixture

## Commands

```text
cargo fmt --check
git diff --check -- src/lib.rs src/am/mod.rs src/am/ec_spire/mod.rs src/am/ec_spire/root/diagnostics.rs src/am/ec_spire/root/types.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md
cargo check --no-default-features --features pg18
cargo check --no-default-features --features "pg18 pg_test"
cargo pgrx test pg18 test_ec_spire_boundary_replica_identity_snapshot_global_ids
git diff --check -- src/lib.rs src/am/mod.rs src/am/ec_spire/mod.rs src/am/ec_spire/root/diagnostics.rs src/am/ec_spire/root/types.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md
```

## Key Results

- `cargo fmt --check`: passed.
- `git diff --check`: passed for the code/task files and again after the design
  doc follow-up.
- `cargo check --no-default-features --features pg18`: passed.
- `cargo check --no-default-features --features "pg18 pg_test"`: passed.
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_identity_snapshot_global_ids`: passed.

No benchmark or latency measurement is claimed by this packet.
