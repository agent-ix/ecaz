# Task 28 IVF vacuum posting-page compaction

## Scope

This packet covers commit `2c1196c2` (`ivf: compact vacuumed posting pages`).

The change moves IVF vacuum deletion from unconditional no-compact deletion to page compaction where it is safe:

- Posting-only blocks use `PageIndexMultiDelete(...)`, reclaiming tuple storage for later posting inserts.
- Blocks protected by persisted directory item pointers keep `PageIndexTupleDeleteNoCompact(...)`, preserving directory tuple offsets.

This keeps the existing streaming vacuum shape: `bulkdelete_list_postings` walks and rewrites posting blocks incrementally through `rewrite_ivf_postings_for_list_blocks`; it does not materialize a full posting list in memory.

## Why the directory-block guard exists

Small indexes can place directory tuples and posting tuples on the same block. Compacting such a mixed block can move directory line pointers and invalidate persisted directory `ItemPointer`s. The implementation passes the current directory block as a no-compact guard while allowing compaction on posting-only blocks.

## Validation

Focused PG18 tests run:

- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `git diff --check`

All passed.

## Remaining A3 Work

This is page-local posting tuple compaction, not full relation truncation. It improves reuse of vacuumed posting pages, but it does not yet truncate empty trailing relation pages. A larger churn measurement packet is still needed before claiming index-size convergence under sustained insert/delete load.
