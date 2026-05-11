---
head_sha: 685ee19b73ab342bede114b83e377dc9412fee26
packet: 30785-spire-stage-e-local-cancel
timestamp: 2026-05-11T00:21:27Z
---

# Artifact Manifest

- Packet/topic: `30785-spire-stage-e-local-cancel`
- Lane: Phase 11 Stage E local multi-instance fault matrix
- Fixture: `local_cancel`
- Storage format: transport probe only; no index storage asserted
- Rerank mode: not applicable
- Surface: one coordinator PG18 cluster plus one remote PG18 cluster addressed
  through two remote descriptor entries
- Isolated one-index-per-table vs shared-table: not applicable; transport
  fault fixture does not build remote indexes
- Head SHA: `685ee19b73ab342bede114b83e377dc9412fee26`

## Command

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case local_cancel \
  --artifact-dir review/30785-spire-stage-e-local-cancel/artifacts \
  --run-id 30785
```

## Artifacts

- `stage_e_fault_local_cancel.log`
  - Key line: `stage_e_fault_local_cancel_passed=true`
- `stage_e_fault_local_cancel_strict.log`
  - Matrix row:
    `local_cancel,local_query_cancelled,cancel_query,remote_executor_cancelled,cancel_query,remote_executor_cancelled,cancelled_dispatch_count=fanout; retained_candidate_batch_count=0`
  - Key raw rows:
    `2,remote_transport_failed,local_query_cancelled,0` and
    `3,remote_transport_failed,local_query_cancelled,0`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_query_cancelled,0,none,remote_executor_cancellation,remote_executor_cancelled`
- `stage_e_fault_local_cancel_degraded.log`
  - Matrix row:
    `local_cancel,local_query_cancelled,cancel_query,remote_executor_cancelled,cancel_query,remote_executor_cancelled,cancelled_dispatch_count=fanout; retained_candidate_batch_count=0`
  - Key raw rows:
    `2,remote_transport_failed,local_query_cancelled,0` and
    `3,remote_transport_failed,local_query_cancelled,0`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_query_cancelled,0,none,remote_executor_cancellation,remote_executor_cancelled`
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
- `cargo test production_executor_transport_local_statement_timeout_cancels_all_dispatches --no-default-features --features pg18`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
