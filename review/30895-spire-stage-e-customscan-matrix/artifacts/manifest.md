# Artifact Manifest: 30895 SPIRE Stage E CustomScan Matrix

- head SHA: `5c5d35eb494b76b1386d71b8b445b44626b2e19e`
- packet/topic: `30895-spire-stage-e-customscan-matrix`
- timestamp: 2026-05-12 UTC
- storage format: RaBitQ
- rerank mode: default fixture scoring
- isolated/shared surface: isolated local PG18 clusters per fixture; separate
  coordinator/remote clusters for the CustomScan read proof and Stage E
  multicluster fixtures

## CustomScan Read Proof

- artifact: `multicluster-customscan-read.log`
- driver artifact: `customscan-read-driver.log`
- command:
  `scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/30895-spire-stage-e-customscan-matrix/artifacts`
- key result lines:
  - `Custom Scan (EcSpireDistributedScan) on ec_spire_customscan_coord_sql`
  - `read_row=10,remote alpha`
  - `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`

## Fault Matrix

Command family:
`bash scripts/run_spire_multicluster_stage_e_*_pg18.sh --case <case> --skip-install --artifact-dir <case-dir> --run-dir <short-target-dir>`

Each case records a top-level smoke log plus strict/degraded logs named
`stage_e_fault_<case>_strict.log` and
`stage_e_fault_<case>_degraded.log`.
PostgreSQL server logs are included in each case directory for debugging, but
the per-case smoke and strict/degraded logs are the cited source of truth.

- `fault-simulated-network-partition/`: `stage_e_fault_simulated_network_partition_passed=true`
- `fault-epoch_mismatch/`: `stage_e_fault_epoch_mismatch_passed=true`
- `fault-version_skew/`: `stage_e_fault_version_skew_passed=true`
- `fault-fingerprint_mismatch/`: `stage_e_fault_fingerprint_mismatch_passed=true`
- `fault-missing_or_reindexed_remote_index/`: `stage_e_fault_missing_or_reindexed_remote_index_passed=true`
- `fault-connection_reset_mid_batch/`: `stage_e_fault_connection_reset_mid_batch_passed=true`
- `fault-local_cancel/`: `stage_e_fault_local_cancel_passed=true`
- `fault-local_statement_timeout/`: `stage_e_fault_local_statement_timeout_passed=true`
- `fault-remote_backend_termination/`: `stage_e_fault_remote_backend_termination_passed=true`
- `fault-remote_oom/`: `stage_e_fault_remote_oom_passed=true`
- `fault-remote_statement_timeout/`: `stage_e_fault_remote_statement_timeout_passed=true`

## Lifecycle Matrix

Command family:
`bash scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh --case <case> --skip-install --artifact-dir <case-dir> --run-dir <short-target-dir>`

Each case records a top-level smoke log plus strict/degraded logs named
`stage_e_lifecycle_<case>_strict.log` and
`stage_e_lifecycle_<case>_degraded.log`.
The lifecycle driver was rerun with short `target/se95-lc-*` run directories to
keep PostgreSQL Unix socket paths below the 107-byte limit; the per-case logs
below are the cited source of truth.

- `lifecycle-create_index_concurrently_missing_descriptor/`: `stage_e_lifecycle_create_index_concurrently_missing_descriptor_passed=true`
- `lifecycle-create_index_concurrently_new_descriptor/`: `stage_e_lifecycle_create_index_concurrently_new_descriptor_passed=true`
- `lifecycle-drop_remote_index_before_fanout/`: `stage_e_lifecycle_drop_remote_index_before_fanout_passed=true`
- `lifecycle-drop_remote_index_in_flight/`: `stage_e_lifecycle_drop_remote_index_in_flight_passed=true`
- `lifecycle-reindex_remote_index_before_fanout/`: `stage_e_lifecycle_reindex_remote_index_before_fanout_passed=true`
- `lifecycle-reindex_remote_index_in_flight/`: `stage_e_lifecycle_reindex_remote_index_in_flight_passed=true`

## Static Check

- artifact: `git-diff-check.log`
- command: `git diff --check`
- result: passed
