# Review Request: Task 111c Page-Scatter Heap-TID Decode Optimization

## Scope

This packet is a narrow 111c optimization checkpoint after packet 002 showed that TQ page scatter was correct and zero-copy but slower than the copy fallback.

Code under review:

- `5e43d2c2f9d4edc99dbf8f597dd467bb0a8c965a` (`Task 111c: avoid heap tid allocation in page scatter`)

## What Changed

- `src/am/ec_ivf/page.rs`
  - adds `IvfColumnarFrozenListPinnedPages::extend_heap_tids_into`, which decodes heap TIDs from the pinned page bytes directly into a caller-provided scratch vector.
- `src/am/ec_ivf/scan.rs`
  - changes `IvfBorrowedPostingScratch::push_columnar_posting` to use the append helper instead of allocating a fresh `Vec<ItemPointer>` per posting and then copying it into scratch.
  - preserves count validation and truncates the scratch vector on mismatch before returning an error.

## Validation

Artifacts are under `reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/`.

- `cargo-pgrx-test-pg18-page-scatter-equivalence.log`
  - `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
  - `1 passed; 0 failed; 2130 filtered out`
- `cargo-build-release-pg18.log`
  - `Finished release profile [optimized] target(s) in 5m 45s`
- `suite-status.log`
  - `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

## EXPLAIN A/B Result

Fixture: local PG18, database `task111b_columnar_bench_r2`, isolated Task 111b 50k TQ columnar index, nprobe 32, rerank off, shared hits warm and reads zero in both cells.

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Page scatter after fix | 0 | 18,358,272 | 0 | 31,649 | 35.775 |
| Copy fallback same head | 18,887,163 | 0 | 18,358,272 | 16,589 | 20.720 |
| Page scatter packet 002 r3 baseline | 0 | 18,358,272 | 0 | 46,983 | 51.210 |

Interpretation:

- This removes a real overhead source and improves page scatter substantially versus packet 002.
- It still does not make page scatter competitive with copy fallback; the next hot spots are likely per-posting page/metadata lookups and generic borrowed batch construction.
- This remains a Task 111c iteration checkpoint, not a promotion/latency-win packet.

## Review Focus

- Is `extend_heap_tids_into` the right ownership boundary for avoiding per-posting heap-TID allocation?
- Is the scratch rollback on decoded-count mismatch sufficient?
- Any concern keeping the existing `heap_tids(index) -> Vec<ItemPointer>` method for non-hot callers/tests while the borrowed scan path uses the allocation-free helper?
