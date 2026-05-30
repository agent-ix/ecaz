# Review Request: Task 68 Zero-Replica Fast Path Measurement

## Scope

This packet measures code commit
`c8f98a71da07e8d1417642fcbbe558ce0ae942d9`, which skips boundary rerouting in
`build_recursive_leaf_rows_by_pid` when `boundary_replica_count=0`.

The suite reuses the existing Task 68 PG18 fixture database and measures one
index per table:

- 10k: `task68_spire_10k_load_corpus`, `nlists=32`
- 100k: `task68_spire_100k_load_corpus`, `nlists=128`

Both use `storage_format='turboquant'`, `recursive_fanout=8`,
`nprobe=24`, `rerank_width=25`, and top graph enabled.

## Result

The fast path removes the measured 100k draft leaf-row hotspot:

| fixture | baseline | fast path | result |
| --- | ---: | ---: | --- |
| 10k total | 806 ms | 372 ms | 2.17x faster |
| 100k total | 22482 ms | 3362 ms | 6.69x faster |
| 100k `draft_leaf_rows_ms` | 19182 ms | 25 ms | 99.9 % lower |
| 100k `draft_total_ms` | 19248 ms | 92 ms | 99.5 % lower |

The packet-local suite completed cleanly:

```text
[suite:task68-spire-zero-replica-fast-path-measurement] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Evidence

- Suite config: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite.json`
- Manifest: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/manifest.md`
- Summary: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/measurement-summary.md`
- 10k log: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/create-10k-spire-fastpath-index.log`
- 100k log: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/create-100k-spire-fastpath-index.log`

## Reviewer Ask

Please check that this packet is enough evidence for landing the first Phase 2
P0 slice and that the updated ranking is defensible: after this slice, the
largest remaining measured 100k phases are heap scan, top graph/object-store,
top-level assignment, and top-level k-means. Task 69 already covers the shared
training/assignment parallelism lane, so the remaining Task 68-specific
candidate is top graph/object-store.
