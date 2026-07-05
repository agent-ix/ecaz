# Review Request: Task 68 Top-Graph Cache Measurement

## Scope

This packet measures code commit
`fe7d5e6892dc1e7154eb95d8e620b22bef070d10`, which caches pairwise
centroid distances for SPIRE top-graph construction.

The suite repeats the same 10k and 100k Task 68 build split used by packet
005. Both builds use `storage_format='turboquant'`,
`boundary_replica_count=0`, top graph enabled, `recursive_fanout=8`,
`nprobe=24`, and `rerank_width=25`.

## Result

Against packet 005's post-fast-path baseline:

| fixture | baseline total_ms | cache total_ms | total delta | baseline top_graph_ms | cache top_graph_ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 372 | 384 | +3.2 % | 59 | 68 |
| 100k | 3362 | 3236 | -3.7 % | 935 | 847 |

Suite status:

```text
[suite:task68-spire-top-graph-cache-measurement] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Interpretation

The cache produced a real 100k win, but it did not clear the Task 68 Phase 2
`~5 % of total build wall time at 100k` gate: `126 ms / 3362 ms = 3.7 %`.

I am therefore treating this as a modest measured improvement and shelving
additional top-graph work for this task unless review says to continue deeper
into the Vamana builder. The remaining large phases are heap scan
(`1234 ms`), top graph (`847 ms`), assignment (`573 ms`), and k-means
(`489 ms`); heap scan is PG callback input collection and Task 69 already
covers the shared training/assignment lane.

## Evidence

- Suite config: `reviews/task-68/007-top-graph-cache-measurement/artifacts/suite.json`
- Manifest: `reviews/task-68/007-top-graph-cache-measurement/artifacts/manifest.md`
- Summary: `reviews/task-68/007-top-graph-cache-measurement/artifacts/measurement-summary.md`
- 10k log: `reviews/task-68/007-top-graph-cache-measurement/artifacts/create-10k-spire-topgraph-cache-index.log`
- 100k log: `reviews/task-68/007-top-graph-cache-measurement/artifacts/create-100k-spire-topgraph-cache-index.log`

## Reviewer Ask

Please confirm whether this measured but sub-gate win should stay landed as a
small constant-factor cleanup, and whether shelving further top-graph work is
the right Task 68 closeout call.
