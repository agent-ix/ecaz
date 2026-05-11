# Artifact Manifest: SPIRE Stage E Lifecycle Reindex In Flight

Head SHA: `83560b38db9a6b405692b6b1093c2d6c7e4d63a2`
Packet: `30792-spire-stage-e-lifecycle-reindex-in-flight`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle matrix
- Fixture: local one-coordinator / one-remote PG18 multi-instance run
- Lifecycle case: `reindex_remote_index_in_flight`
- Storage format: `rabitq`
- Rerank mode: production candidate receive helper, top-k 1
- Surface: isolated one-index-per-table fixture tables
- Command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case reindex_remote_index_in_flight --artifact-dir review/30792-spire-stage-e-lifecycle-reindex-in-flight/artifacts --run-id 30792b
```

## Artifacts

### `stage_e_lifecycle_reindex_remote_index_in_flight.log`

Full fixture stdout/stderr, including extension install output and strict /
degraded lifecycle summaries.

Key result lines:

```text
stage_e_lifecycle_reindex_remote_index_in_flight_passed=true
SPIRE Stage E lifecycle reindex_remote_index_in_flight PG18 fixture passed
```

### `stage_e_lifecycle_reindex_remote_index_in_flight_strict.log`

Strict-mode lifecycle result.

Key result lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_remote_identity_before_drop=32636e33464a95d5
remote_reindexed_identity=4d87893355fdc4c5
expected_status=remote_candidate_receive_failed
expected_candidate_receive_failed_dispatch_count=1
expected_first_candidate_receive_failure_category=endpoint_identity_mismatch
expected_next_executor_step=compact_candidate_receive
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

### `stage_e_lifecycle_reindex_remote_index_in_flight_degraded.log`

Degraded-mode lifecycle result.

Key result lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_remote_identity_before_drop=4d8aef335600a7ee
remote_reindexed_identity=4d79f13355f23821
expected_status=degraded_ready
expected_degraded_skipped_dispatch_count=1
expected_first_degraded_skip_category=endpoint_identity_mismatch
expected_next_executor_step=remote_heap_resolution
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,endpoint_identity_mismatch,remote_heap_resolution,degraded_ready
```

### `remote-ready-postgres.log`

Remote PostgreSQL server log for the lifecycle fixture.

### `coord-postgres.log`

Coordinator PostgreSQL server log for the lifecycle fixture.
