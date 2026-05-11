# Artifact Manifest

- head_sha: `5bb76a036e5b1baa2dcb16ae2c06efbff2cde243`
- packet: `30784-spire-stage-e-remote-backend-termination`
- timestamp: `2026-05-11T00:08:40Z`
- lane: Stage E local multi-instance fault matrix
- fixture: `remote_backend_termination`
- storage_format: n/a transport probe
- rerank_mode: none
- surface: coordinator plus ready remote PG18 clusters; production transport probe uses one `pg_terminate_backend(pg_backend_pid())` request and one ready request
- isolated_one_index_per_table: n/a transport probe
- shared_table_surface: no
- command:
  `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case remote_backend_termination --artifact-dir review/30784-spire-stage-e-remote-backend-termination/artifacts --run-id 30784`

## Artifacts

- `stage_e_fault_remote_backend_termination.log`
  - Full fixture stdout/stderr.
  - Key line: `stage_e_fault_remote_backend_termination_passed=true`
- `stage_e_fault_remote_backend_termination_strict.log`
  - Strict-mode backend termination evidence.
  - Key raw rows: `2,remote_transport_failed,remote_backend_terminated,0` and `3,ready,none,3`
  - Key summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_backend_terminated,1,0,none,production_transport_adapter,remote_transport_failed`
- `stage_e_fault_remote_backend_termination_degraded.log`
  - Degraded-mode backend termination evidence.
  - Key raw rows: `2,remote_transport_failed,remote_backend_terminated,0` and `3,ready,none,3`
  - Key summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_backend_terminated,compact_candidate_receive,requires_compact_candidate_receive`
- `remote-ready-postgres.log`
  - Ready remote PostgreSQL log for the fixture run.
- `coord-postgres.log`
  - Coordinator PostgreSQL log for the fixture run.

## Regression

- The shared transport fixture was also rerun for
  `remote_statement_timeout` with `--run-id 30783r --skip-install`.
  The pass marker was `stage_e_fault_remote_statement_timeout_passed=true`.
