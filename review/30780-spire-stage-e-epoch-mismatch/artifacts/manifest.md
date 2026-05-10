# Packet 30780 Artifact Manifest

- code head SHA: `53e62148067fbcf6f05f2b350f8a8fb90f63357a`
- task-note SHA: `5e62299913aa299c63e9711606918f07f3904c1a`
- packet/topic: `30780-spire-stage-e-epoch-mismatch`
- lane: Stage E fault matrix runtime fixture
- fixture: one coordinator, one ready remote, one stale-epoch remote descriptor
- fault case: `epoch_mismatch`
- storage format: SPIRE `ecvector_spire_ip_ops`, `nlists = 2`, PG18 pg_test build
- rerank mode: pre-dispatch executor-state summary only, no heap rerank/materialization
- surface shape: isolated one-index-per-table coordinator and ready remote; separate strict/degraded coordinator indexes
- timestamp: 2026-05-10 16:14 PDT
- command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case epoch_mismatch \
  --artifact-dir review/30780-spire-stage-e-epoch-mismatch/artifacts \
  --run-id 30780
```

## Artifacts

- `stage_e_fault_epoch_mismatch.log`
  - full fixture stdout/stderr, including pg_test install, strict row,
    degraded row, and pass signal.
  - key result lines:
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_epoch_window,stale_epoch`
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,stale_epoch,production_transport_adapter,requires_production_transport_adapter`
    - `stage_e_fault_epoch_mismatch_passed=true`
    - `SPIRE Stage E epoch_mismatch PG18 fixture passed`
- `stage_e_fault_epoch_mismatch_strict.log`
  - strict-mode expected and observed executor-state summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_epoch_window,stale_epoch`
- `stage_e_fault_epoch_mismatch_degraded.log`
  - degraded-mode expected and observed executor-state summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,stale_epoch,production_transport_adapter,requires_production_transport_adapter`
- `coord-postgres.log`
  - coordinator PostgreSQL log from the successful run.
- `remote-ready-postgres.log`
  - ready remote PostgreSQL log from the successful run.

## Scope Boundary

This packet covers only the `epoch_mismatch` Stage E fault row. Remaining
Stage E fault and lifecycle rows still need packet-local logs.
