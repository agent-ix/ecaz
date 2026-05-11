# Review Request: SPIRE Stage E Remote OOM

## Summary

This packet adds Stage E runtime evidence for the `remote_oom` row in
`ec_spire_remote_search_stage_e_fault_matrix()`.

The code checkpoint is `04ed2d9f0ba4038455d522f168ff64c5c3056c02`.

## What Changed

- Added `remote_oom` to the Stage E transport fixture family.
- The fault remote raises SQLSTATE `53200` to simulate an out-of-memory remote
  query failure.
- The coordinator-visible transport row exposes only the normalized
  `remote_query_failed` category, not the raw remote error text.
- Extended `ecaz dev spire-multicluster fault-pg18 --case remote_oom`.
- Updated the Phase 11 task with packet `30788`.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case remote_oom \
  --artifact-dir review/30788-spire-stage-e-remote-oom/artifacts \
  --run-id 30788
```

Strict mode:

```text
observed_transport_rows=2,remote_transport_failed,remote_query_failed,0
3,ready,none,3
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,remote_query_failed,1,0,none,production_transport_adapter,remote_transport_failed
```

Degraded mode:

```text
observed_transport_rows=2,remote_transport_failed,remote_query_failed,0
3,ready,none,3
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_query_failed,compact_candidate_receive,requires_compact_candidate_receive
```

Pass marker:

```text
stage_e_fault_remote_oom_passed=true
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Is SQLSTATE `53200` an acceptable local fixture for the Stage E remote-OOM
  row without forcing an actual memory exhaustion event?
- Does the evidence prove sanitized coordinator categorization well enough?
- Is strict/degraded state handling aligned with the Stage E fault matrix?
