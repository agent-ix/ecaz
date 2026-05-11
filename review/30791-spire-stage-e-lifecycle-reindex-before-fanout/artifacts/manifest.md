# Artifact Manifest: SPIRE Stage E Lifecycle Reindex Before Fanout

Head SHA: `8bb2fc83ba827492c6431afa9c5ba48de6d9721a`
Packet: `30791-spire-stage-e-lifecycle-reindex-before-fanout`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle matrix
- Fixture: local one-coordinator / one-remote PG18 multi-instance run
- Lifecycle case: `reindex_remote_index_before_fanout`
- Storage format: `rabitq`
- Rerank mode: production candidate receive helper, top-k 1
- Surface: isolated one-index-per-table fixture tables
- Command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case reindex_remote_index_before_fanout --artifact-dir review/30791-spire-stage-e-lifecycle-reindex-before-fanout/artifacts --run-id 30791c
```

## Artifacts

### `stage_e_lifecycle_reindex_remote_index_before_fanout.log`

Full fixture stdout/stderr, including extension install output and strict /
degraded lifecycle summaries.

Key result lines:

```text
stage_e_lifecycle_reindex_remote_index_before_fanout_passed=true
SPIRE Stage E lifecycle reindex_remote_index_before_fanout PG18 fixture passed
```

### `stage_e_lifecycle_reindex_remote_index_before_fanout_strict.log`

Strict-mode lifecycle result.

Key result lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx before fanout
dropped_remote_identity_before_drop=323e0c33462ad312
remote_reindexed_identity=32417233462db63b
expected_status=remote_candidate_receive_failed
expected_candidate_receive_failed_dispatch_count=1
expected_first_candidate_receive_failure_category=endpoint_identity_mismatch
expected_next_executor_step=compact_candidate_receive
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

### `stage_e_lifecycle_reindex_remote_index_before_fanout_degraded.log`

Degraded-mode lifecycle result.

Key result lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx before fanout
dropped_remote_identity_before_drop=323e0c33462ad312
remote_reindexed_identity=32417233462db63b
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
