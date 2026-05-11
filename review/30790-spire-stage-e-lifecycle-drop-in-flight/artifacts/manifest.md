# Artifact Manifest: SPIRE Stage E Lifecycle Drop In Flight

Head SHA: `83983d9a867936e80663855b53b228522286d71e`
Packet: `30790-spire-stage-e-lifecycle-drop-in-flight`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle matrix
- Fixture: local one-coordinator / one-remote PG18 multi-instance run
- Lifecycle case: `drop_remote_index_in_flight`
- Storage format: `rabitq`
- Rerank mode: production candidate receive helper, top-k 1
- Surface: isolated one-index-per-table fixture tables
- Command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case drop_remote_index_in_flight --artifact-dir review/30790-spire-stage-e-lifecycle-drop-in-flight/artifacts --run-id 30790
```

## Artifacts

### `stage_e_lifecycle_drop_remote_index_in_flight.log`

Full fixture stdout/stderr, including extension install output and strict /
degraded lifecycle summaries.

Key result lines:

```text
stage_e_lifecycle_drop_remote_index_in_flight_passed=true
SPIRE Stage E lifecycle drop_remote_index_in_flight PG18 fixture passed
```

### `stage_e_lifecycle_drop_remote_index_in_flight_strict.log`

Strict-mode lifecycle result.

Key result lines:

```text
injection=DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_index_to_regclass_is_null=t
expected_status=remote_candidate_receive_failed
expected_candidate_receive_failed_dispatch_count=1
expected_first_candidate_receive_failure_category=remote_index_unavailable
expected_next_executor_step=compact_candidate_receive
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

### `stage_e_lifecycle_drop_remote_index_in_flight_degraded.log`

Degraded-mode lifecycle result.

Key result lines:

```text
injection=DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_index_to_regclass_is_null=t
expected_status=degraded_ready
expected_degraded_skipped_dispatch_count=1
expected_first_degraded_skip_category=remote_index_unavailable
expected_next_executor_step=remote_heap_resolution
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready
```

### `remote-ready-postgres.log`

Remote PostgreSQL server log for the lifecycle fixture.

### `coord-postgres.log`

Coordinator PostgreSQL server log for the lifecycle fixture.
