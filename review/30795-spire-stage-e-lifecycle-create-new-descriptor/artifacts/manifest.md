# Artifact Manifest: SPIRE Stage E Lifecycle Create New Descriptor

Head SHA: `b440b8863422604a1fbe96e039647781ea7262d3`
Packet: `30795-spire-stage-e-lifecycle-create-new-descriptor`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle matrix
- Fixture: local one-coordinator / one-remote PG18 multi-instance run
- Lifecycle case: `create_index_concurrently_new_descriptor`
- Storage format: `rabitq`
- Rerank mode: production candidate receive helper, top-k 1
- Surface: isolated one-index-per-table fixture tables
- Command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case create_index_concurrently_new_descriptor --artifact-dir review/30795-spire-stage-e-lifecycle-create-new-descriptor/artifacts --run-id 30795d
```

## Artifacts

### `stage_e_lifecycle_create_index_concurrently_new_descriptor.log`

Full fixture stdout/stderr, including extension install output and strict /
degraded lifecycle summaries.

Key result lines:

```text
stage_e_lifecycle_create_index_concurrently_new_descriptor_passed=true
SPIRE Stage E lifecycle create_index_concurrently_new_descriptor PG18 fixture passed
```

### `stage_e_lifecycle_create_index_concurrently_new_descriptor_strict.log`

Strict-mode lifecycle result.

Key result lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_new_descriptor_strict_idx after request construction before receive; register descriptor_generation=11 before receive
old_descriptor_identity=326008334647b2ac
new_descriptor_identity=32636e33464a95d5
expected_status=requires_remote_heap_resolution
expected_candidate_receive_ready_dispatch_count=2
expected_candidate_receive_failed_dispatch_count=0
observed_descriptor_row=2,11,ec_spire_stage_e_lifecycle_new_descriptor_strict_idx,32636e33464a95d5,active
observed_summary=spire_remote_fanout_executor_v1,2,2,2,0,none,2,0,none,remote_heap_resolution,requires_remote_heap_resolution
```

### `stage_e_lifecycle_create_index_concurrently_new_descriptor_degraded.log`

Degraded-mode lifecycle result.

Key result lines:

```text
injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_new_descriptor_degraded_idx after request construction before receive; register descriptor_generation=21 before receive
old_descriptor_identity=4d87893355fdc4c5
new_descriptor_identity=4d84233355fae19c
expected_status=requires_remote_heap_resolution
expected_candidate_receive_ready_dispatch_count=2
expected_candidate_receive_failed_dispatch_count=0
observed_descriptor_row=2,21,ec_spire_stage_e_lifecycle_new_descriptor_degraded_idx,4d84233355fae19c,active
observed_summary=spire_remote_fanout_executor_v1,2,2,2,0,none,2,0,none,remote_heap_resolution,requires_remote_heap_resolution
```

### `remote-ready-postgres.log`

Remote PostgreSQL server log for the lifecycle fixture.

### `coord-postgres.log`

Coordinator PostgreSQL server log for the lifecycle fixture.
