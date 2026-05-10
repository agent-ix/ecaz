# Artifact Manifest

- head_sha: `6ee441d653ed70d368135bb4682175692e1ae224`
- packet: `30781-spire-stage-e-missing-remote-index`
- timestamp: `2026-05-10T23:48:24Z`
- lane: Stage E local multi-instance fault matrix
- fixture: `missing_or_reindexed_remote_index`
- storage_format: `rabitq`
- rerank_mode: none
- surface: coordinator plus ready remote PG18 clusters; candidate-receive helper uses a missing-index conninfo on the ready remote and a ready loopback candidate batch on the coordinator
- isolated_one_index_per_table: yes
- shared_table_surface: no
- command:
  `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case missing_or_reindexed_remote_index --artifact-dir review/30781-spire-stage-e-missing-remote-index/artifacts --run-id 30781l --skip-install`

## Artifacts

- `stage_e_fault_missing_or_reindexed_remote_index.log`
  - Full fixture stdout/stderr.
  - Key line: `stage_e_fault_missing_or_reindexed_remote_index_passed=true`
- `stage_e_fault_missing_or_reindexed_remote_index_strict.log`
  - Strict-mode candidate receive evidence.
  - Key raw rows: `2,remote_candidate_receive_failed,remote_index_unavailable,0` and `3,ready,none,1`
  - Key summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed`
- `stage_e_fault_missing_or_reindexed_remote_index_degraded.log`
  - Degraded-mode candidate receive evidence.
  - Key raw rows: `2,remote_candidate_receive_failed,remote_index_unavailable,0` and `3,ready,none,1`
  - Key summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready`
- `remote-ready-postgres.log`
  - Ready remote PostgreSQL log for the final fixture run.
- `coord-postgres.log`
  - Coordinator PostgreSQL log for the final fixture run.
