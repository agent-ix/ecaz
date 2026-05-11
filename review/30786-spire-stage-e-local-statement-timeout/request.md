# Review Request: SPIRE Stage E Local Statement Timeout

## Summary

This packet adds Stage E runtime evidence for the `local_statement_timeout` row
in `ec_spire_remote_search_stage_e_fault_matrix()`.

The code checkpoint is `d097ffdb31207113aafa5faf41b72cec9d9708fd`.

## What Changed

- Added SQL-visible pg-test fixture helpers that trigger PostgreSQL's
  statement-timeout indicator in the coordinator backend and then exercise the
  production transport adapter's interrupt polling path.
- Extended `ecaz dev spire-multicluster fault-pg18 --case
  local_statement_timeout`.
- Kept the local statement-timeout behavior query-wide: every in-flight remote
  dispatch is cancelled, no remote batches are retained, and the exact first
  cancellation category remains `local_statement_timeout`.
- Updated the Phase 11 task with packet `30786`.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case local_statement_timeout \
  --artifact-dir review/30786-spire-stage-e-local-statement-timeout/artifacts \
  --run-id 30786
```

Strict mode:

```text
observed_transport_rows=2,remote_transport_failed,local_statement_timeout,0
3,remote_transport_failed,local_statement_timeout,0
observed_summary=spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_statement_timeout,0,none,remote_executor_cancellation,remote_executor_cancelled
```

Degraded mode:

```text
observed_transport_rows=2,remote_transport_failed,local_statement_timeout,0
3,remote_transport_failed,local_statement_timeout,0
observed_summary=spire_remote_fanout_executor_v1,2,0,0,0,0,2,local_statement_timeout,0,none,remote_executor_cancellation,remote_executor_cancelled
```

Pass marker:

```text
stage_e_fault_local_statement_timeout_passed=true
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo test production_executor_transport_local_statement_timeout_cancels_all_dispatches --no-default-features --features pg18`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Does the fixture prove the production PostgreSQL statement-timeout interrupt
  bridge, rather than the timer-based local-cancel test primitive?
- Is it correct that strict and degraded mode both report query-wide
  `remote_executor_cancelled` for local statement timeout?
- Are the shortened SQL helper names acceptable as pg-test-only fixture
  surfaces?
