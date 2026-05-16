# Artifact Manifest: SPIRE Stage E Lifecycle Create Missing Descriptor

Head SHA: `7a930be1bd206e5023f2ee8529f00e6035bdd2d1`
Packet: `30793-spire-stage-e-lifecycle-create-missing-descriptor`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle matrix
- Fixture: local one-coordinator / one-remote PG18 multi-instance run
- Lifecycle case: `create_index_concurrently_missing_descriptor`
- Storage format: `rabitq`
- Rerank mode: production executor state summary, top-k 1
- Surface: isolated one-index-per-table fixture tables
- Command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case create_index_concurrently_missing_descriptor --artifact-dir review/30793-spire-stage-e-lifecycle-create-missing-descriptor/artifacts --run-id 30793e
```

## Artifacts

### `stage_e_lifecycle_create_index_concurrently_missing_descriptor.log`

Full fixture stdout/stderr, including extension install output and strict /
degraded lifecycle summaries.

Key result lines:

```text
stage_e_lifecycle_create_index_concurrently_missing_descriptor_passed=true
SPIRE Stage E lifecycle create_index_concurrently_missing_descriptor PG18 fixture passed
```

### `stage_e_lifecycle_create_index_concurrently_missing_descriptor_strict.log`

Strict-mode lifecycle result.

Key result lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx before descriptor registration
created_index_to_regclass_is_not_null=t
expected_status=requires_remote_node_descriptor
expected_planned_dispatch_count=0
expected_blocked_before_dispatch_count=1
expected_degraded_skipped_dispatch_count=0
expected_next_executor_step=remote_node_descriptor
observed_request_readiness_rows=local,0,active,ready,ready
remote,2,missing,requires_remote_node_descriptor,requires_remote_node_descriptor
observed_summary=spire_remote_fanout_executor_v1,1,0,1,1,0,1,0,0,0,0,none,remote_node_descriptor,requires_remote_node_descriptor
```

### `stage_e_lifecycle_create_index_concurrently_missing_descriptor_degraded.log`

Degraded-mode lifecycle result.

Key result lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx before descriptor registration
created_index_to_regclass_is_not_null=t
expected_status=degraded_skipped
expected_planned_dispatch_count=1
expected_blocked_before_dispatch_count=0
expected_degraded_skipped_dispatch_count=1
expected_first_degraded_skip_category=requires_remote_node_descriptor
expected_next_executor_step=remote_heap_resolution
observed_request_readiness_rows=local,0,active,ready,ready
remote,2,missing,requires_remote_node_descriptor,requires_remote_node_descriptor
observed_summary=spire_remote_fanout_executor_v1,1,1,0,1,1,0,0,0,0,1,requires_remote_node_descriptor,remote_heap_resolution,degraded_skipped
```

### `remote-ready-postgres.log`

Remote PostgreSQL server log for the lifecycle fixture.

### `coord-postgres.log`

Coordinator PostgreSQL server log for the lifecycle fixture.
