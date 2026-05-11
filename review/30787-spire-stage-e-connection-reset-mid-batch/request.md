# Review Request: SPIRE Stage E Connection Reset Mid Batch

## Summary

This packet adds Stage E runtime evidence for the
`connection_reset_mid_batch` row in
`ec_spire_remote_search_stage_e_fault_matrix()`.

The code checkpoint is `523b51aac9f01f78bb077f64c15b559c1b32f3de`.

## What Changed

- Added `connection_reset_mid_batch` to the Stage E transport fixture family.
- The fault SQL attempts to start a result stream and then terminates the
  remote backend in the same query:
  `generate_series_first_row_then_pg_terminate_backend`.
- Extended `ecaz dev spire-multicluster fault-pg18 --case
  connection_reset_mid_batch`.
- Updated the Phase 11 task with packet `30787`.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case connection_reset_mid_batch \
  --artifact-dir review/30787-spire-stage-e-connection-reset-mid-batch/artifacts \
  --run-id 30787
```

Strict mode:

```text
observed_transport_rows=2,remote_transport_failed,remote_backend_terminated,0
3,ready,none,3
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,remote_backend_terminated,1,0,none,production_transport_adapter,remote_transport_failed
```

Degraded mode:

```text
observed_transport_rows=2,remote_transport_failed,remote_backend_terminated,0
3,ready,none,3
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_backend_terminated,compact_candidate_receive,requires_compact_candidate_receive
```

Pass marker:

```text
stage_e_fault_connection_reset_mid_batch_passed=true
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Does this fixture sufficiently distinguish mid-batch connection reset from
  the simpler `remote_backend_termination` case?
- Is `remote_backend_terminated` the right normalized category for a remote
  connection closing while a result stream is in flight?
- Is strict/degraded state handling aligned with the Stage E fault matrix?
