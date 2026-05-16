# Review Request: SPIRE Stage E Lifecycle Create Missing Descriptor

## Summary

This packet adds runtime evidence for the Stage E lifecycle row
`create_index_concurrently_missing_descriptor`.

Code checkpoint: `7a930be1bd206e5023f2ee8529f00e6035bdd2d1`

The fixture creates a remote `ec_spire` index with `CREATE INDEX CONCURRENTLY`
before registering a coordinator descriptor for that remote node. It then
rewrites one coordinator placement to the remote node and verifies the
pre-dispatch descriptor plane: strict mode fails closed at
`requires_remote_node_descriptor`, while degraded mode skips the node before
any conninfo secret lookup, socket open, or endpoint identity probe.

This slice also adjusts pg_test-only SPIRE debug manifest rewrite helpers to
load coordinator fanout manifests, so lifecycle fixtures can mutate coordinator
manifests that legitimately contain remote placements without bypassing the
production local heap delivery guard.

## Evidence

Artifacts are stored under `artifacts/`:

- `stage_e_lifecycle_create_index_concurrently_missing_descriptor.log`
- `stage_e_lifecycle_create_index_concurrently_missing_descriptor_strict.log`
- `stage_e_lifecycle_create_index_concurrently_missing_descriptor_degraded.log`
- `remote-ready-postgres.log`
- `coord-postgres.log`
- `manifest.md`

Strict mode key lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx before descriptor registration
created_index_to_regclass_is_not_null=t
observed_request_readiness_rows=local,0,active,ready,ready
remote,2,missing,requires_remote_node_descriptor,requires_remote_node_descriptor
observed_summary=spire_remote_fanout_executor_v1,1,0,1,1,0,1,0,0,0,0,none,remote_node_descriptor,requires_remote_node_descriptor
```

Degraded mode key lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx before descriptor registration
created_index_to_regclass_is_not_null=t
observed_request_readiness_rows=local,0,active,ready,ready
remote,2,missing,requires_remote_node_descriptor,requires_remote_node_descriptor
observed_summary=spire_remote_fanout_executor_v1,1,1,0,1,1,0,0,0,0,1,requires_remote_node_descriptor,remote_heap_resolution,degraded_skipped
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/debug.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case create_index_concurrently_missing_descriptor --artifact-dir review/30793-spire-stage-e-lifecycle-create-missing-descriptor/artifacts --run-id 30793e`

## Review Focus

- Whether the missing-descriptor lifecycle row proves the pre-dispatch
  descriptor registration boundary for a concurrently-created remote index.
- Whether strict and degraded outcomes should both remain zero for secret,
  socket, and endpoint identity probes when no remote descriptor exists.
- Whether the pg_test-only debug helper loader change is properly scoped to
  coordinator manifest mutation without weakening production local heap scans.
