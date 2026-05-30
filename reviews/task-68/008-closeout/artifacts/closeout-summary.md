# Task 68 Closeout Summary

## Packet Chain

- `001-spire-build-timing-notices`: initial build timing instrumentation,
  approved with measurement-shape notes.
- `002-characterization`: Phase 1 ranking direction approved; reviewer asked
  for drilldown before flipping the gate fully green.
- `003-timing-drilldown`: Phase 1 blockers closed; reviewer approved Phase 2
  P0 list.
- `004-zero-replica-leaf-row-fast-path`: code approved.
- `005-zero-replica-fast-path-measurement`: measured win approved.
- `006-top-graph-distance-cache`: code shape approved; measurement owed.
- `007-top-graph-cache-measurement`: measured 3.7% 100k win, below the 5%
  continuation gate; awaiting reviewer confirmation to shelve deeper top-graph
  work.
- `008-closeout`: this final measurement and closeout request.

## Final Build Split

The closeout suite repeated the Phase 1 split on both fixture sizes using
`boundary_replica_count=0`, top graph enabled, `recursive_fanout=8`,
`storage_format='turboquant'`, and isolated closeout indexes.

| fixture | total_ms | heap_scan_ms | kmeans_ms | assignment_ms | draft_leaf_rows_ms | top_graph_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 338 | 138 | 148 | 15 | 1 | 24 |
| 100k | 3418 | 1307 | 490 | 574 | 20 | 946 |

The second same-seed determinism builds came in at 10k `308 ms` and 100k
`2950 ms`; all structural hashes matched the first build.

The structural-hash equality is the determinism gate. The wall-time variance
between first and second builds is cache/JIT/host noise and does not affect the
determinism claim.

## Exit Criteria Check

- Phase 1 characterization: satisfied by reviewer approval in packet 003.
- P0 slices: zero-replica leaf row fast path landed and was approved in
  packets 004/005; top-graph distance cache landed with a measured 100k win but
  was below the continuation gate in packet 007; heap scan is PostgreSQL input
  collection; shared k-means/assignment work is owned by closed Task 69.
- Final measurement: satisfied by `suite-manifest.json`,
  `build-and-compare-10k.log`, and `build-and-compare-100k.log`.
- Recall floor: 10k `recall@10=0.9995`; 100k `recall@10=0.8525` at valid
  `nprobe=16` on 200 queries. No pre-Task-68 same-config SPIRE 100k
  comparator is on file, so this packet records the post-Task-68 baseline; the
  preservation gate is satisfied by the code-review argument that Task 68 did
  not change scoring semantics, plus same-seed structural equality for leaf
  assignments and packet 006's byte-equivalent top-graph cache test.
- Determinism: hierarchy, root routing, routing centroids, leaf summary, and
  leaf assignments all hash-equal across same-seed duplicate builds for 10k and
  100k.
- Safety: no new `unsafe { ... }` blocks were introduced by the Task 68 code
  slices.

## Closeout Position

Task 68 has delivered the dominant SPIRE-specific win:

```text
100k pre-fast-path baseline total_ms=22482
100k post-fast-path measurement total_ms=3362
100k closeout first-build total_ms=3418
```

The remaining large costs are not attractive Task 68 continuation slices:
heap scan is PG callback input collection, shared training/assignment moved to
Task 69, and deeper top-graph work was measured below the task's continuation
gate after the small cache cleanup.
