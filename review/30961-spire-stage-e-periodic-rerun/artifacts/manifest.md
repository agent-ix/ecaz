# Artifact Manifest: SPIRE Stage E Periodic Rerun

- Head SHA: `1939007c2813bcd14ff782c064190e6060bba1ef`
- Packet/topic: `30961-spire-stage-e-periodic-rerun`
- Lane / fixture / storage format / rerank mode: Phase 12.7 Stage E periodic
  rerun; local PG18 multi-cluster CustomScan read, fault, and lifecycle
  fixtures; RaBitQ SPIRE indexes; default fixture scoring/rerank path.
- Surface isolation: isolated local PG18 clusters per fixture using separate
  coordinator/remote data directories and packet-local artifact directories.
  The matrix does not use shared-table multi-index surfaces beyond each
  fixture's local catalogs.
- Timestamp: `2026-05-13T03:56:04Z`

## Setup Notes

The packet includes two setup retry logs for transparency:

- `customscan-read/`: failed before fixture execution because a relative
  `--run-dir` made PostgreSQL try to create the socket lock below the data
  directory.
- `customscan-read-retry/`: failed before fixture execution inside the sandbox
  because PostgreSQL could not bind Unix-domain sockets.

These two directories are not cited as validation evidence. The successful
CustomScan proof is `customscan-read-success/`.

## CustomScan Read Proof

### `customscan-read-success/multicluster-customscan-read.log`

- Command:
  `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install --artifact-dir /home/peter/dev/ecaz/review/30961-spire-stage-e-periodic-rerun/artifacts/customscan-read-success --run-dir /home/peter/dev/ecaz/target/se30961-customscan-read-success`
- Key result lines:
  `Custom Scan (EcSpireDistributedScan) on ec_spire_customscan_coord_sql`
  `read_row=10,remote alpha`
  `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`
  `SPIRE multicluster CustomScan read passed`

## Fault Matrix

Each fault case records a top-level smoke log plus strict/degraded logs in its
case directory.

- `fault-simulated-network-partition/`:
  `stage_e_fault_simulated_network_partition_passed=true`
- `fault-epoch_mismatch/`:
  `stage_e_fault_epoch_mismatch_passed=true`
- `fault-version_skew/`:
  `stage_e_fault_version_skew_passed=true`
- `fault-fingerprint_mismatch/`:
  `stage_e_fault_fingerprint_mismatch_passed=true`
- `fault-missing_or_reindexed_remote_index/`:
  `stage_e_fault_missing_or_reindexed_remote_index_passed=true`
- `fault-connection_reset_mid_batch/`:
  `stage_e_fault_connection_reset_mid_batch_passed=true`
- `fault-local_cancel/`:
  `stage_e_fault_local_cancel_passed=true`
- `fault-local_statement_timeout/`:
  `stage_e_fault_local_statement_timeout_passed=true`
- `fault-remote_backend_termination/`:
  `stage_e_fault_remote_backend_termination_passed=true`
- `fault-remote_oom/`:
  `stage_e_fault_remote_oom_passed=true`
- `fault-remote_statement_timeout/`:
  `stage_e_fault_remote_statement_timeout_passed=true`

## Lifecycle Matrix

Each lifecycle case records a top-level smoke log plus strict/degraded logs in
its case directory.

- `lifecycle-create_index_concurrently_missing_descriptor/`:
  `stage_e_lifecycle_create_index_concurrently_missing_descriptor_passed=true`
- `lifecycle-create_index_concurrently_new_descriptor/`:
  `stage_e_lifecycle_create_index_concurrently_new_descriptor_passed=true`
- `lifecycle-drop_remote_index_before_fanout/`:
  `stage_e_lifecycle_drop_remote_index_before_fanout_passed=true`
- `lifecycle-drop_remote_index_in_flight/`:
  `stage_e_lifecycle_drop_remote_index_in_flight_passed=true`
- `lifecycle-reindex_remote_index_before_fanout/`:
  `stage_e_lifecycle_reindex_remote_index_before_fanout_passed=true`
- `lifecycle-reindex_remote_index_in_flight/`:
  `stage_e_lifecycle_reindex_remote_index_in_flight_passed=true`

## Static Check

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 1939007c2813bcd14ff782c064190e6060bba1ef^ 1939007c2813bcd14ff782c064190e6060bba1ef" review/30961-spire-stage-e-periodic-rerun/artifacts/git-diff-check.log`
- Result:
  `Script done on 2026-05-12 20:57:33-07:00 [COMMAND_EXIT_CODE="0"]`
