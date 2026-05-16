# Artifact Manifest

- head_sha: `60a0e21d796b024d9d70e440b80e44c6e2cc6a17`
- packet: `30783-spire-stage-e-remote-statement-timeout`
- timestamp: `2026-05-10T23:59:24Z`
- lane: Stage E local multi-instance fault matrix
- fixture: `remote_statement_timeout`
- storage_format: n/a transport probe
- rerank_mode: none
- surface: coordinator plus ready remote PG18 clusters; production transport probe uses one slow `pg_sleep(0.30)` request under `ec_spire.remote_search_statement_timeout_ms=25` and one ready request
- isolated_one_index_per_table: n/a transport probe
- shared_table_surface: no
- command:
  `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case remote_statement_timeout --artifact-dir review/30783-spire-stage-e-remote-statement-timeout/artifacts --run-id 30783 --skip-install`

## Artifacts

- `stage_e_fault_remote_statement_timeout.log`
  - Full fixture stdout/stderr.
  - Key line: `stage_e_fault_remote_statement_timeout_passed=true`
- `stage_e_fault_remote_statement_timeout_strict.log`
  - Strict-mode transport timeout evidence.
  - Key raw rows: `2,remote_transport_failed,remote_statement_timeout,0` and `3,ready,none,3`
  - Key summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_statement_timeout,1,0,none,production_transport_adapter,remote_transport_failed`
- `stage_e_fault_remote_statement_timeout_degraded.log`
  - Degraded-mode transport timeout evidence.
  - Key raw rows: `2,remote_transport_failed,remote_statement_timeout,0` and `3,ready,none,3`
  - Key summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_statement_timeout,compact_candidate_receive,requires_compact_candidate_receive`
- `remote-ready-postgres.log`
  - Ready remote PostgreSQL log for the fixture run.
- `coord-postgres.log`
  - Coordinator PostgreSQL log for the fixture run.
