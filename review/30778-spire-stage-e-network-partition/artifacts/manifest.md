# Packet 30778 Artifact Manifest

- head SHA: `1cb6153745fdf9a0fb2e7eb3aa2024c217fcf650`
- task-note SHA: `57316c3d8b18ed33dc36fb4f84b2a6a6c4296e53`
- packet/topic: `30778-spire-stage-e-network-partition`
- lane: Stage E fault matrix runtime fixture
- fixture: one coordinator, one ready remote, one unreachable conninfo
- storage format: SPIRE `ecvector_spire_ip_ops`, `nlists = 2`, PG18 pg_test build
- rerank mode: transport-summary only, no heap rerank/materialization
- surface shape: isolated one-index-per-table coordinator and ready remote
- timestamp: 2026-05-10 15:55 PDT
- command:

```text
target/debug/ecaz dev spire-multicluster fault-pg18 \
  --case simulated_network_partition \
  --artifact-dir review/30778-spire-stage-e-network-partition/artifacts \
  --run-id 30778e
```

## Artifacts

- `stage_e_fault_simulated_network_partition.log`
  - full fixture stdout/stderr, including pg_test install, strict row,
    degraded row, and pass signal.
  - key result lines:
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,connect_failed,1,0,none,production_transport_adapter,remote_transport_failed`
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,connect_failed,compact_candidate_receive,requires_compact_candidate_receive`
    - `stage_e_fault_simulated_network_partition_passed=true`
    - `SPIRE Stage E simulated network partition PG18 fixture passed`
- `stage_e_fault_simulated_network_partition_strict.log`
  - strict-mode expected and observed transport summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,connect_failed,1,0,none,production_transport_adapter,remote_transport_failed`
- `stage_e_fault_simulated_network_partition_degraded.log`
  - degraded-mode expected and observed transport summary.
  - key result line:
    - `observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,connect_failed,compact_candidate_receive,requires_compact_candidate_receive`
- `coord-postgres.log`
  - coordinator PostgreSQL log from the successful run.
- `remote-ready-postgres.log`
  - ready remote PostgreSQL log from the successful run.

## Scope Boundary

This packet covers only the `simulated_network_partition` Stage E fault row.
Remaining Stage E fault and lifecycle rows still need packet-local logs.
