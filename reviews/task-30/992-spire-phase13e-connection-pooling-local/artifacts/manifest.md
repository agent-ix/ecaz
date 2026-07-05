# Artifact Manifest

Head SHA: `402b92943a5a14149ee956b9cfbbdb2408c95fe5`

Task bucket: `reviews/task-30/992-spire-phase13e-connection-pooling-local`

Timestamp: `2026-05-26T18:42:35Z`

Lane: local PG18, no AWS execution

Fixture: local AWS-shape 1 coordinator plus 3 remotes, plus Phase 13e local gate suite

Storage format: `rabitq`

Rerank mode: default / no rerank override

Surface: mixed isolated remote tables for AWS-shape harness and established local gate fixtures; no shared production AWS resources

## Artifacts

### `phase13e-aws-harness-local.log`

Command:

```bash
bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --artifact-dir reviews/task-30/992-spire-phase13e-connection-pooling-local/artifacts
```

Key result lines:

- `published_static_remote_placements`
- `Custom Scan (EcSpireDistributedScan)`
- `remote_fanout: 3`
- `result_source remote_heap_candidates`
- `status ready`
- `socket_open_count 3` for the direct cold production read profile
- benchmark production read rows with `socket_open_sum 0`
- `HARNESS PASSED`

### `smoke-customscan-read.log`

Command: produced by the local AWS-shape harness smoke step.

Key result lines:

- `Custom Scan (EcSpireDistributedScan)`
- `remote_fanout: 3`
- `tuple_transport_status: ready`
- `result_source remote_heap_candidates`
- `final_heap_fetch_status remote_ready`
- `status ready`

### `production-read-profile-smoke.log`

Command: produced by the local AWS-shape harness production profile step.

Key result lines:

- `dispatch_count 3`
- `remote_heap_ready_dispatch_count 3`
- `remote_heap_failed_dispatch_count 0`
- `result_source remote_heap_candidates`
- `socket_open_count 3`

### `bench-spire-pipeline-smoke.log`

Command: produced by the local AWS-shape harness benchmark step.

Key result lines:

- production read profile rows are `ready`
- `result_source remote_heap_candidates`
- `dispatch_sum 15`
- `socket_open_sum 0`
- `candidate_query_sum 15`
- `heap_query_sum 15`

### `local-gates-after-pooling/phase13e-local-gates-summary.tsv`

Command:

```bash
bash scripts/run_spire_phase13e_local_gates_pg18.sh --suite all --artifact-dir reviews/task-30/992-spire-phase13e-connection-pooling-local/artifacts/local-gates-after-pooling
```

Key result lines:

- all 22 rows have status `pass`
- `phase13e_local_gates_passed suite=all`

### `local-gates-after-pooling/phase13e-static-remote-placement/phase13e-static-remote-placement.log`

Command: produced by the Phase 13e local gate suite.

Key result lines:

- `placement_summary=2:1,3:1,4:1`
- `profile_summary=ready|3|3|3|3|6`
- `remote_fanout: 3`
- read rows match exact rows
- `transport_overlap` timeline summary recorded candidate and heap overlap

### `local-gates-after-pooling/transport-overlap/multicluster-transport-overlap.log`

Command: produced by the Phase 13e local gate suite.

Key result lines:

- `fast_completed_before_slow=true`
- `SPIRE multicluster PG18 transport overlap passed`

