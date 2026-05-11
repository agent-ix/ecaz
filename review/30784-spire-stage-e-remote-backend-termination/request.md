# Review Request: SPIRE Stage E Remote Backend Termination

## Summary

This packet covers the Stage E `remote_backend_termination` runtime row.

The implementation extends the transport-fault fixture family from packet
30783 to support backend termination:

- `ecaz dev spire-multicluster fault-pg18 --case remote_backend_termination`
- pg-test transport probe case helpers for fixed transport fault SQL snippets
- strict/degraded summary assertions for `remote_backend_terminated`

The fixture drives the production transport probe helper with one request that
terminates its own backend and one ready remote request.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case remote_backend_termination \
  --artifact-dir review/30784-spire-stage-e-remote-backend-termination/artifacts \
  --run-id 30784
```

Strict mode:

- terminated node: `remote_transport_failed,remote_backend_terminated,0`
- ready node: `ready,none,3`
- summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_backend_terminated,1,0,none,production_transport_adapter,remote_transport_failed`

Degraded mode:

- terminated node: `remote_transport_failed,remote_backend_terminated,0`
- ready node: `ready,none,3`
- summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_backend_terminated,compact_candidate_receive,requires_compact_candidate_receive`

The shared fixture was rerun for `remote_statement_timeout` with
`--run-id 30783r --skip-install`; it still passed after the shared transport
helper changes.

Artifacts are listed in `artifacts/manifest.md`.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/cli.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Does the transport case helper stay narrow enough for fixture evidence?
- Does backend termination map to the intended `remote_backend_terminated` category?
- Are strict fail-closed and degraded skip assertions aligned with the Stage E matrix row?
