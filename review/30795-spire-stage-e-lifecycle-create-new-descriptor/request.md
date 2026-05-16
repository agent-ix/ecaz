# Review Request: SPIRE Stage E Lifecycle Create New Descriptor

## Summary

This packet adds runtime evidence for the Stage E lifecycle row
`create_index_concurrently_new_descriptor`.

Code checkpoint: `b440b8863422604a1fbe96e039647781ea7262d3`

The fixture builds production candidate receive requests against an old remote
descriptor, then injects `CREATE INDEX CONCURRENTLY` and advances the
coordinator descriptor generation before receive. The already-planned receive
continues against the old descriptor/index in both strict and degraded modes,
while the descriptor table shows the newer descriptor generation has been
registered for a later query.

The observed candidate-receive status is `requires_remote_heap_resolution`
because this fixture stops at the candidate receive handoff; that is the
receive-stage proof of the lifecycle matrix's `ready` action.

## Evidence

Artifacts are stored under `artifacts/`:

- `stage_e_lifecycle_create_index_concurrently_new_descriptor.log`
- `stage_e_lifecycle_create_index_concurrently_new_descriptor_strict.log`
- `stage_e_lifecycle_create_index_concurrently_new_descriptor_degraded.log`
- `remote-ready-postgres.log`
- `coord-postgres.log`
- `manifest.md`

Strict mode key lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_new_descriptor_strict_idx after request construction before receive; register descriptor_generation=11 before receive
old_descriptor_identity=326008334647b2ac
new_descriptor_identity=32636e33464a95d5
observed_descriptor_row=2,11,ec_spire_stage_e_lifecycle_new_descriptor_strict_idx,32636e33464a95d5,active
observed_summary=spire_remote_fanout_executor_v1,2,2,2,0,none,2,0,none,remote_heap_resolution,requires_remote_heap_resolution
```

Degraded mode key lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_new_descriptor_degraded_idx after request construction before receive; register descriptor_generation=21 before receive
old_descriptor_identity=4d87893355fdc4c5
new_descriptor_identity=4d84233355fae19c
observed_descriptor_row=2,21,ec_spire_stage_e_lifecycle_new_descriptor_degraded_idx,4d84233355fae19c,active
observed_summary=spire_remote_fanout_executor_v1,2,2,2,0,none,2,0,none,remote_heap_resolution,requires_remote_heap_resolution
```

## Validation

- `cargo fmt --check`
- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case create_index_concurrently_new_descriptor --artifact-dir review/30795-spire-stage-e-lifecycle-create-new-descriptor/artifacts --run-id 30795d`

## Review Focus

- Whether the pg_test helper correctly models request construction before
  descriptor generation advancement and receive after the advancement.
- Whether `requires_remote_heap_resolution` is the correct receive-layer proof
  for the lifecycle matrix's `ready` action.
- Whether this completes the Stage E lifecycle runtime evidence set.
