# Packet 30779 Artifact Manifest

- code head SHA: `760d47c3b86e012340bc5fdc1aa88fb2d5dac8e9`
- task-note SHA: `7099b9665a0a9797f22b7099db61a2e8286b77c5`
- packet/topic: `30779-spire-stage-e-version-skew`
- lane: Stage E fault matrix runtime fixture
- fixture: one coordinator, one ready remote, one incompatible-version remote descriptor
- fault case: `version_skew`
- storage format: SPIRE `ecvector_spire_ip_ops`, `nlists = 2`, PG18 pg_test build
- rerank mode: pre-dispatch executor-state summary only, no heap rerank/materialization
- surface shape: isolated one-index-per-table coordinator and ready remote; separate strict/degraded coordinator indexes
- timestamp: 2026-05-10 16:08 PDT
- command:

```text
target/debug/ecaz dev spire-multicluster fault-pg18 \
  --case version_skew \
  --artifact-dir review/30779-spire-stage-e-version-skew/artifacts \
  --run-id 30779b
```

## Artifacts

- `stage_e_fault_version_skew.log`
  - full fixture stdout/stderr, including pg_test install, strict row,
    degraded row, and pass signal.
  - key result lines:
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_extension_version,incompatible_extension_version`
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,incompatible_extension_version,production_transport_adapter,requires_production_transport_adapter`
    - `stage_e_fault_version_skew_passed=true`
    - `SPIRE Stage E version_skew PG18 fixture passed`
- `stage_e_fault_version_skew_strict.log`
  - strict-mode expected and observed executor-state summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_extension_version,incompatible_extension_version`
- `stage_e_fault_version_skew_degraded.log`
  - degraded-mode expected and observed executor-state summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,incompatible_extension_version,production_transport_adapter,requires_production_transport_adapter`
- `coord-postgres.log`
  - coordinator PostgreSQL log from the successful run.
- `remote-ready-postgres.log`
  - ready remote PostgreSQL log from the successful run.

## Scope Boundary

This packet covers only the `version_skew` Stage E fault row. Remaining Stage E
fault and lifecycle rows still need packet-local logs.
