# Review Request: Benign No-Op Vacuum Callbacks

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `ambulkdelete` and `amvacuumcleanup` no longer hard-error for `tqhnsw`.
- Both callbacks now return a valid `IndexBulkDeleteResult` without reclaiming tuples or mutating index pages.
- Reported stats currently include main-fork page count and an exact count of element tuples.

Review focus:
- Whether this no-op vacuum behavior is semantically safe for the current live-insert/build state
- Accuracy and safety of the returned `IndexBulkDeleteResult`
- Any hidden assumptions around dead tuples, deleted heap rows, or executor/vacuum expectations

Questions to answer:
- Is reporting exact element-tuple count with zero removals the right current contract?
- Is there any vacuum caller expectation we still violate by leaving stale tuples untouched?
- Are there missing tests around repeated vacuum calls or fully empty indexes?

---

## Review Comments

Status at `803bf5e`:
- Questions above: closed for this stage as addressed or skipped/not needed.
- Comments 1-5 and 8: not needed. These review notes validate the current no-op vacuum contract rather than identifying changes required in this stage.
- Comment 6 addressed by adding empty-index vacuum coverage.
- Comment 7 addressed by adding repeated-vacuum coverage.

### 1. The no-op vacuum is safe for the current state

Since the scan path hard-errors (`amrescan`/`amgettuple` call `unsupported_build_only_error`), no query executor will ever try to use this index for tuple visibility decisions. The index is effectively write-only. Leaving dead heap-TIDs in element tuples has no observable effect on query results because no query can read them. This is a sound contract for now.

### 2. `ambulkdelete` ignores the callback — this is correct but has a subtle implication

At line 252-259, `ambulkdelete` ignores the `callback` and `callback_state` parameters entirely. Normally, the AM is supposed to call `callback(heap_tid, callback_state)` for each index tuple to ask whether the corresponding heap row is dead. By skipping this, vacuum will never learn that the index "acknowledged" dead rows. 

PostgreSQL's vacuum will still reclaim heap tuples regardless — the `IndexBulkDeleteCallback` is informational for the AM, not a gate for heap cleanup. So this is safe. However, `num_index_tuples` in the result will include element tuples that point to deleted heap rows, which means `pg_stat_user_indexes.idx_tup_read` and friends may overcount. This is cosmetic for now.

### 3. `tqhnsw_noop_vacuum_stats` reuses the incoming `stats` pointer correctly

At line 851-854, if `stats` is null (which happens on the first `ambulkdelete` call), a new `IndexBulkDeleteResult` is allocated via `PgBox::alloc0()`. If non-null (as when `amvacuumcleanup` receives stats from `ambulkdelete`), it reuses the pointer. The `alloc0` ensures all numeric fields start at zero. This is correct.

### 4. `count_element_tuples` full scan is O(n) per vacuum — acceptable for now

`count_element_tuples` (line 869-915) scans every data page to count elements with tag `TQ_ELEMENT_TAG`. This runs during vacuum, which is a background maintenance operation, so the cost is acceptable. The function takes shared locks on each buffer and releases them promptly.

### 5. Potential issue: `count_element_tuples` counts elements on the page that `ambulkdelete` is currently processing

Since both `ambulkdelete` and `amvacuumcleanup` call `tqhnsw_noop_vacuum_stats`, and `count_element_tuples` acquires `BUFFER_LOCK_SHARE`, there's no lock conflict — the no-op vacuum never holds exclusive locks on data pages. No issue here.

### 6. Missing test: vacuum on an empty index

The existing test `test_tqhnsw_vacuum_callbacks_are_benign_noops` (lib.rs:1453) creates 3 rows, builds an index, deletes one row, then vacuums. This is good.

**Missing:** A test that vacuums an index with no data pages at all (built on an empty table via `ambuildempty`). `count_element_tuples` would see `block_count <= FIRST_DATA_BLOCK_NUMBER` and return 0, which should work. But verifying this with a test would confirm that `debug_vacuum_stats` doesn't crash when iterating over zero data pages.

### 7. Missing test: repeated vacuum calls

The test runs one vacuum cycle. A test that runs `VACUUM` twice on the same index would confirm that the stats remain stable and that no state is corrupted across repeated no-op vacuum passes. This is low risk but would increase confidence.

### 8. `estimated_count = false` is the right choice

At line 862, reporting exact counts (not estimates) is correct since the implementation does a full scan. PostgreSQL uses this flag to decide whether to trust the tuple count for planning — since scans are disabled via infinite cost estimates, this doesn't affect query plans, but it's still the honest answer.
