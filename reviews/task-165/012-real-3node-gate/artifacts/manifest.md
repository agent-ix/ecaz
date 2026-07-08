# Manifest — Task 165 packet 012 (real 3-instance multi-node gate)

- **head SHA:** ac42b707a
- **task bucket / packet:** reviews/task-165/012-real-3node-gate
- **branch:** task-165-ec-distann-m3
- **date:** 2026-07-08
- **surface:** THREE real PG18 instances (separate data dirs / sockets / ports
  39710–39712), one index per node, release `.so` from the pgrx-install lib dir.
  NOT loopback — the coordinator's CustomScan issues real cross-process
  `expand` + `materialize_row_payloads` calls to the other instances.
- **distribution:** replicated deterministic corpus (identical in-SQL generation
  + identical insertion order ⇒ identical local-mode vec_ids + identical
  seed-deterministic global graph on every node); roster partitions ownership of
  serving. Non-standard vs a disjoint-shard build — stated because build-global-
  then-distribute tooling does not yet exist; the replicated model is the
  correct-recall real-multinode substrate available today and yields an exact
  (byte-identical) recall oracle.

## Command

```
ecaz dev distann-multicluster local-multinode-pg18 \
  --nodes 3 --rows <2000|50000> --dim <16|32> --queries 50 --top-k 10 \
  --artifact-dir reviews/task-165/012-real-3node-gate/artifacts[/50k]
```

## Key result lines

- 2k / dim 16 (`artifacts/distann-multinode-summary.log`):
  `RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0`,
  `fault_drill dead_remote_port fail_closed=true`.
- 50k / dim 32 (`artifacts/50k/distann-multinode-summary.log`):
  `RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0`,
  `fault_drill dead_remote_port fail_closed=true`.

distinct_recall(multinode) − distinct_recall(single) = 0 ≥ −0.001 at both scales.

## Not in this packet (open)

- Suite-driven (`ecaz bench suite`) form of the recall gate against the
  kept-running coordinator (matches 006-P1's letter; the identity proof here is
  strictly stronger).
- Full TC-042 fault taxonomy + FR-082 lifecycle / epoch-swap-under-load (Slice C).
