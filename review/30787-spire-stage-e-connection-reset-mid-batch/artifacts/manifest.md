---
head_sha: 523b51aac9f01f78bb077f64c15b559c1b32f3de
packet: 30787-spire-stage-e-connection-reset-mid-batch
timestamp: 2026-05-11T00:34:47Z
---

# Artifact Manifest

- Packet/topic: `30787-spire-stage-e-connection-reset-mid-batch`
- Lane: Phase 11 Stage E local multi-instance fault matrix
- Fixture: `connection_reset_mid_batch`
- Storage format: transport probe only; no index storage asserted
- Rerank mode: not applicable
- Surface: one coordinator PG18 cluster plus one remote PG18 cluster addressed
  through two remote descriptor entries
- Isolated one-index-per-table vs shared-table: not applicable; transport
  fault fixture does not build remote indexes
- Head SHA: `523b51aac9f01f78bb077f64c15b559c1b32f3de`

## Command

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case connection_reset_mid_batch \
  --artifact-dir review/30787-spire-stage-e-connection-reset-mid-batch/artifacts \
  --run-id 30787
```

## Artifacts

- `stage_e_fault_connection_reset_mid_batch.log`
  - Key line: `stage_e_fault_connection_reset_mid_batch_passed=true`
- `stage_e_fault_connection_reset_mid_batch_strict.log`
  - Matrix row:
    `connection_reset_mid_batch,remote_backend_terminated,fail_closed,remote_transport_failed,skip_node,degraded_skipped,transport_failed_dispatch_count+1; degraded_skipped_dispatch_count+1`
  - Key raw rows:
    `2,remote_transport_failed,remote_backend_terminated,0` and
    `3,ready,none,3`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,2,1,1,remote_backend_terminated,1,0,none,production_transport_adapter,remote_transport_failed`
- `stage_e_fault_connection_reset_mid_batch_degraded.log`
  - Matrix row:
    `connection_reset_mid_batch,remote_backend_terminated,fail_closed,remote_transport_failed,skip_node,degraded_skipped,transport_failed_dispatch_count+1; degraded_skipped_dispatch_count+1`
  - Key raw rows:
    `2,remote_transport_failed,remote_backend_terminated,0` and
    `3,ready,none,3`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_backend_terminated,compact_candidate_receive,requires_compact_candidate_receive`
- `remote-ready-postgres.log`
  - Remote PostgreSQL server log for the fixture.
- `coord-postgres.log`
  - Coordinator PostgreSQL server log for the fixture.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
