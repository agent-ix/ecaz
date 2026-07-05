# Review Request: Task 28 IVF Insert Hot-Path Narrowing

## Summary

This packet records the follow-up to packet 30056's live-insert finding. Commit
`bfbb40d` changes `ec_ivf` live insert so duplicate heap-TID rejection scans
only the assigned list instead of every IVF list. The full all-list check
remains available through the debug validation helper.

The change preserves the existing duplicate rejection regression and materially
improves the synthetic insert-stress surface.

## Measurement

Same fixture shape as packet 30056: local PG18 scratch, synthetic 4D
`ecvector`, 1000 seed rows, `nlists=16`, `nprobe=16`, `batch_rows=1`,
10-second insert phase.

| packet | check scope | concurrency | inserted rows | rows/s | final live rows | index bytes |
|---|---|---:|---:|---:|---:|---:|
| 30056 | all lists | 1 | 668 | 66.80 | 1668 | 237568 |
| 30057 | assigned list | 1 | 2753 | 275.30 | 3753 | 393216 |
| 30056 | all lists | 4 | 1592 | 159.20 | 2592 | 311296 |
| 30057 | assigned list | 4 | 6575 | 657.50 | 7575 | 819200 |

That is about a 4.1x throughput improvement for both the 1-worker and
4-worker points on this local synthetic harness. The 4-worker point remains
about 2.39x the 1-worker point, so metadata/list update contention and
per-insert centroid loading are still likely bottlenecks.

## Interpretation

This closes the first obvious live-insert hot-path issue from packet 30056.
The next insert slice should profile or remove the remaining fixed per-row
work:

- centroid model reload per insert;
- one posting per row with no live-insert coalescing;
- list-directory and metadata counter writes per inserted row.

DiskANN remains task 29 and is not included.
