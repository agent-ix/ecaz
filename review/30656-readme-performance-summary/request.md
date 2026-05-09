# Review Request: README Performance Summary

## Summary

The README performance section now presents storage-format tradeoffs separately
from a single access-method snapshot covering HNSW, IVF, and DiskANN.

Code checkpoints:

- `d5a738b2` (`Clarify README performance summary`)
- `f71ed012` (`Document quantized vector byte sizes`)

## Scope

- Replaces the previous HNSW-vs-DiskANN block plus detached IVF block with one
  "Index Family Snapshot" table covering `ec_hnsw`, `ec_ivf`, and
  `ec_diskann`.
- Adds an IVF storage-format table grounded in the Task 28 A10 closure packet,
  showing `turboquant`, `pq_fastscan`, and `rabitq` recall/latency/index-size
  tradeoffs on 10K and 25K matched-width surfaces.
- Expands the compression table with current 1536-dimensional per-vector code
  sizes for `pq_fastscan` g8 and IVF `rabitq`, while calling out that full index
  size also includes access-method, codebook, graph/list, and rerank overhead.
- Keeps the current Apple M5 IVF balanced/quality anchors from Task 31 and the
  HNSW/DiskANN local PG18 anchors from Task 29d.

## Sources

- `review/11109-task29d-final-readiness/`
- `review/30145-task28-ivf-a10-current-closure/`
- `review/30203-task31-current-m5-candidate-decision/`

## Validation

- `git diff --check`

No code tests were run. This is a documentation-only checkpoint.
