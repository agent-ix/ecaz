# Review Request: Task 28 IVF Build/Training/Vacuum/Insert Follow-Up

## Summary

This packet closes the reviewer-requested deeper pass over IVF build,
training, vacuum, and live-insert behavior, and adds a checked-in
`ecaz stress ivf-insert` harness for repeatable insert-concurrency
measurements.

The first 1000-row insert stress run found a correctness bug before it could
measure throughput: live insert could append a posting onto a block that also
held list-directory tuples, and the directory traversal then decoded that new
posting as the next directory entry. Commit `43563e5` fixes traversal by
searching for the next directory-tagged tuple instead of assuming the next
physical tuple is always a directory.

## Code Findings

- Training is sampled, not full-corpus k-means. `training_sample_rows=0`
  resolves to `min(row_count, 10000)`, and explicit values are capped at row
  count.
- Build still retains the full heap tuple set in memory, including source
  vectors and payloads, then stages populated pages before flushing. The
  training cliff is bounded, but build memory remains a product-gate risk at
  larger scale.
- Bulk assignment creates per-list tuple-index vectors and writes one posting
  per heap tuple. There is no posting compaction during build.
- Vacuum walks each list, materializes list postings into a `Vec`, rewrites
  postings whose heap TIDs were removed, marks empty postings deleted, and
  repairs list/head/tail and metadata counts. It does not compact deleted
  posting tuples or reclaim posting pages.
- Live insert remains structurally expensive: it loads centroids, scans all
  list postings to reject duplicate heap TIDs, appends one posting, then
  updates both list directory and metadata counters.

## Measurement

The fixed synthetic PG18 insert stress results:

| surface | seed rows | concurrency | duration | inserted rows | rows/s | final live rows | index bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| smoke | 64 | 1 | 2s | 476 | 238.00 | 540 | 98304 |
| c1 | 1000 | 1 | 10s | 668 | 66.80 | 1668 | 237568 |
| c4 | 1000 | 4 | 10s | 1592 | 159.20 | 2592 | 311296 |

The 4-worker point is about 2.38x the 1-worker point, not 4x. That is useful
but not enough to claim a finished concurrency story, especially because this
is only a synthetic 4D insert harness and the installed scratch DB did not
expose the admin snapshot SQL function. The packet therefore reports fallback
relation stats for live row count and index size.

## Interpretation

The reviewer was right to ask for more here. The deeper pass found and fixed a
real live-insert correctness issue, and the remaining performance shape points
to the next concrete work:

- remove the O(total postings) duplicate heap-TID scan from the hot insert
  path, or bound it behind a cheaper correctness guard;
- avoid per-insert full centroid reload once the index metadata/model is stable;
- decide whether metadata `inserted_since_build` needs per-insert persistence
  or can be batched/decontented;
- add a vacuum compaction/reclaim plan before claiming long-lived write-heavy
  IVF indexes are production-ready.

DiskANN remains task 29 and is not included.
