# Review Request: SPIRE Stage E Remote Statement Timeout

## Summary

This packet covers the Stage E `remote_statement_timeout` runtime row.

The implementation adds:

- `ecaz dev spire-multicluster fault-pg18 --case remote_statement_timeout`
- `scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- strict/degraded transport summary assertions for `remote_statement_timeout`

The fixture drives the pg-test production transport probe helper with one slow
remote query under `ec_spire.remote_search_statement_timeout_ms=25` and one
ready remote query.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case remote_statement_timeout \
  --artifact-dir review/30783-spire-stage-e-remote-statement-timeout/artifacts \
  --run-id 30783 \
  --skip-install
```

Strict mode:

- timed-out node: `remote_transport_failed,remote_statement_timeout,0`
- ready node: `ready,none,3`
- summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_statement_timeout,1,0,none,production_transport_adapter,remote_transport_failed`

Degraded mode:

- timed-out node: `remote_transport_failed,remote_statement_timeout,0`
- ready node: `ready,none,3`
- summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_statement_timeout,compact_candidate_receive,requires_compact_candidate_receive`

Artifacts are listed in `artifacts/manifest.md`.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- crates/ecaz-cli/src/cli.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Does the transport fixture model remote statement timeout at the right layer?
- Are strict fail-closed and degraded skip assertions aligned with the Stage E matrix?
- Is a separate transport-fault fixture family the right shape for the remaining transport rows?
