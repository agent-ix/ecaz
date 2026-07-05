# Review Request: Task 41 DiskANN Insert Slot Guards

## Summary

This slice migrates two DiskANN insert-planning heap tuple slots from manual
allocation/drop pairs to `TupleTableSlotGuard::single_for_heap`.

Code commit: `af4cbd5c7a5af5474e8974f1a1564a73a40d411a`

## Changes

Updated `src/am/ec_diskann/routine.rs`:

- The duplicate-probe slot in `ec_diskann_aminsert` now uses
  `TupleTableSlotGuard::single_for_heap`.
- The unique-insert forward-neighbor planning slot now uses
  `TupleTableSlotGuard::single_for_heap`.
- Removed manual `ExecDropSingleTupleTableSlot` calls from the normal path,
  the exact-rerank error path, and the early duplicate-return path.
- Updated `scripts/unsafe_comment_baseline.txt` for line-number movement.

## Baseline

- Before: `4140`
- After: `4140`

The baseline count is unchanged because these callsites sit inside a larger
unsafe AM callback surface, but slot ownership is now centralized in the shared
guard.

## Review Focus

- Confirm slot guard lifetimes cover all uses passed to
  `fetch_heap_source_vector` and `exact_heap_rerank_distance`.
- Confirm early duplicate returns and exact-rerank error paths now drop the
  slot through guard unwinding.
- Confirm the changed allocation failure messages remain specific enough for
  the two insert-planning contexts.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
