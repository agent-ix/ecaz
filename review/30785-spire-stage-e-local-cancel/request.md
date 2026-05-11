# Review Request: SPIRE Stage E Local Cancel

## Summary

This packet adds Stage E runtime evidence for the `local_cancel` row in
`ec_spire_remote_search_stage_e_fault_matrix()`.

The code checkpoint is `685ee19b73ab342bede114b83e377dc9412fee26`.

## What Changed

- Added a test-facing local-cancel transport probe summary so the Stage E
  harness can assert query-wide cancellation state, not only per-row transport
  failures.
- Normalized production executor summary handling so both local cancellation
  categories, `local_query_cancelled` and `local_statement_timeout`, drive the
  same query-wide cancellation path while preserving the distinct first
  cancellation category.
- Extended `ecaz dev spire-multicluster fault-pg18 --case local_cancel`.
- Updated the Phase 11 task with packet `30785`.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case local_cancel \
  --artifact-dir review/30785-spire-stage-e-local-cancel/artifacts \
  --run-id 30785
```

Strict mode:

```text
observed_transport_rows=2,remote_transport_failed,local_query_cancelled,0
3,remote_transport_failed,local_query_cancelled,0
observed_summary=spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_query_cancelled,0,none,remote_executor_cancellation,remote_executor_cancelled
```

Degraded mode:

```text
observed_transport_rows=2,remote_transport_failed,local_query_cancelled,0
3,remote_transport_failed,local_query_cancelled,0
observed_summary=spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_query_cancelled,0,none,remote_executor_cancellation,remote_executor_cancelled
```

Pass marker:

```text
stage_e_fault_local_cancel_passed=true
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo test production_executor_transport_local_statement_timeout_cancels_all_dispatches --no-default-features --features pg18`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Is the local-cancellation summary correctly modeled as query-wide in both
  strict and degraded mode?
- Is it appropriate for `local_statement_timeout` to share the same executor
  cancellation path while preserving its distinct category?
- Is the fixture narrow enough for Stage E without claiming final AM result
  delivery?
