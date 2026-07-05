# Review Request: Task 68 Closeout

## Scope

This packet closes out Task 68 after the approved characterization and landed
P0 slices. It adds no code. The closeout evidence is a single
`ecaz bench suite` packet that repeats the Phase 1 build split, checks
same-seed structural determinism, and runs SPIRE recall@10 on the pinned 10k
and 100k fixtures.

Head SHA: `2c4592b8f9c686ae9b854958674477d7e0d020ac`.

## Result

Suite status:

```text
[suite:task68-spire-build-closeout] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Final build split:

| fixture | total_ms | heap_scan_ms | kmeans_ms | assignment_ms | draft_leaf_rows_ms | top_graph_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 338 | 138 | 148 | 15 | 1 | 24 |
| 100k | 3418 | 1307 | 490 | 574 | 20 | 946 |

Same-seed duplicate builds matched for hierarchy, root routing, routing
centroids, leaf summary, and leaf assignments on both fixture sizes.

Recall floor:

| fixture | nprobe | queries | recall@10 | ndcg@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 16 | 200 | 0.9995 | 1.0000 | 6.37 ms |
| 100k | 16 | 200 | 0.8525 | 0.9835 | 13.78 ms |

`nprobe=16` is used because the closeout indexes have
`top_graph_search_list_size=16`.

## Interpretation

Task 68's dominant SPIRE-specific win is the zero-replica leaf row fast path:

```text
100k pre-fast-path baseline total_ms=22482
100k post-fast-path measurement total_ms=3362
100k closeout first-build total_ms=3418
```

The remaining large 100k costs are heap scan, top graph, assignment, and
k-means. Heap scan is PostgreSQL input collection, shared k-means/assignment is
covered by closed Task 69, and packet 007 measured the top-graph cache as a
real but sub-gate 3.7% win. I recommend shelving deeper top-graph work for Task
68 unless review wants to reopen that lane.

No new `unsafe { ... }` blocks were introduced by the Task 68 slices.

## Evidence

- Suite config: `reviews/task-68/008-closeout/artifacts/suite.json`
- Manifest: `reviews/task-68/008-closeout/artifacts/manifest.md`
- Closeout summary: `reviews/task-68/008-closeout/artifacts/closeout-summary.md`
- Suite manifest: `reviews/task-68/008-closeout/artifacts/suite-manifest.json`
- Suite status: `reviews/task-68/008-closeout/artifacts/suite-status.log`
- Suite report: `reviews/task-68/008-closeout/artifacts/suite-report.log`
- 10k build/determinism log:
  `reviews/task-68/008-closeout/artifacts/build-and-compare-10k.log`
- 100k build/determinism log:
  `reviews/task-68/008-closeout/artifacts/build-and-compare-100k.log`
- 10k recall log:
  `reviews/task-68/008-closeout/artifacts/recall-10k-closeout.log`
- 100k recall log:
  `reviews/task-68/008-closeout/artifacts/recall-100k-closeout.log`

## Reviewer Ask

Please review this as the Task 68 closeout packet. In particular, confirm
whether packet 007's measured but sub-gate top-graph win is enough to shelve
deeper top-graph work and mark Task 68 complete.
