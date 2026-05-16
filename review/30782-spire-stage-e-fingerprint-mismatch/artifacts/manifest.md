# Artifact Manifest

- head_sha: `fd0e66d0d811b2a6f400075e5c11504ba7454dbb`
- packet: `30782-spire-stage-e-fingerprint-mismatch`
- timestamp: `2026-05-10T23:54:15Z`
- lane: Stage E local multi-instance fault matrix
- fixture: `fingerprint_mismatch`
- storage_format: `rabitq`
- rerank_mode: none
- surface: coordinator plus ready remote PG18 clusters; candidate-receive helper uses a deliberately stale/wrong remote index identity for one request and a ready loopback candidate batch for the other
- isolated_one_index_per_table: yes
- shared_table_surface: no
- command:
  `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case fingerprint_mismatch --artifact-dir review/30782-spire-stage-e-fingerprint-mismatch/artifacts --run-id 30782 --skip-install`

## Artifacts

- `stage_e_fault_fingerprint_mismatch.log`
  - Full fixture stdout/stderr.
  - Key line: `stage_e_fault_fingerprint_mismatch_passed=true`
- `stage_e_fault_fingerprint_mismatch_strict.log`
  - Strict-mode candidate receive evidence.
  - Key raw rows: `2,remote_candidate_receive_failed,endpoint_identity_mismatch,0` and `3,ready,none,1`
  - Key summary: `spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed`
- `stage_e_fault_fingerprint_mismatch_degraded.log`
  - Degraded-mode candidate receive evidence.
  - Key raw rows: `2,remote_candidate_receive_failed,endpoint_identity_mismatch,0` and `3,ready,none,1`
  - Key summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,endpoint_identity_mismatch,remote_heap_resolution,degraded_ready`
- `remote-ready-postgres.log`
  - Ready remote PostgreSQL log for the fixture run.
- `coord-postgres.log`
  - Coordinator PostgreSQL log for the fixture run.

## Regression

- The shared candidate-receive fixture was also rerun for
  `missing_or_reindexed_remote_index` with `--run-id 30781r --skip-install`.
  The pass marker was `stage_e_fault_missing_or_reindexed_remote_index_passed=true`.
