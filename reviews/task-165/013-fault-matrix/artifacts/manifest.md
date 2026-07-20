# Manifest — Task 165 packet 013 (TC-042 fault matrix, real 3-instance)

- **head SHA:** (fault-matrix commit on task-165-ec-distann-m3)
- **task bucket / packet:** reviews/task-165/013-fault-matrix
- **branch:** task-165-ec-distann-m3
- **date:** 2026-07-08
- **surface:** three real PG18 instances (ports 39710–39712), replicated
  deterministic corpus (2k rows, dim 16), release `.so`. Same fixture as packet
  012, with the fault matrix enabled.

## Command

```
ecaz dev distann-multicluster local-multinode-pg18 \
  --nodes 3 --rows 2000 --dim 16 --queries 50 --top-k 10 \
  --artifact-dir reviews/task-165/013-fault-matrix/artifacts
```

## Key result lines (distann-multinode-summary.log)

- `RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0`
- `fault_drill simulated_network_partition pass=true`
- `fault_drill epoch_bump_no_false_reject pass=true`
- `fault_drill remote_content_divergence pass=true`
- `fault_drill missing_or_reindexed_remote_index pass=true`
- `fault_drill remote_backend_termination pass=true`
- `recovery ... mismatched_ids=0 recovered=true`

Each fault is classified per NFR-020 (error-or-identical); the command exits
non-zero if any drill fails, the recall gate mismatches, or recovery diverges.

## Not in this packet (Slice C tail, follow-up)

- `remote_statement_timeout`, `hop_round_failure_mid_beam`,
  `missing_node_record`, `placement_drift`, `mid-delete` (FR-083 distributed).
- FR-082 build/publish/retire lifecycle + epoch-swap-under-load.
