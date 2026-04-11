# Review Request: Narrow `tqhnsw_aminsert` Metadata Lock Scope

## Context

Original review packet: `review/10045-insert-metadata-lock-scope/request.md`
Reviewer feedback (open, still worth doing):
`review/10045-insert-metadata-lock-scope/feedback/2026-04-10-01-reviewer.md`

Branch: `fix/10045-insert-metadata-lock-scope`
Off main: `8cd7e71 Move external recall harness truth cache task to tasks folder`
Fix commit: `ea20722 Narrow metadata lock scope in tqhnsw_aminsert`

The 10045 request identified `tqhnsw_aminsert` as the dominant
serialization point for concurrent and bulk inserts: the entire insert
— shape validation, an O(pages) duplicate scan, the append, and a
possible `entry_point` update — ran inside `with_locked_metadata_page`,
holding an exclusive lock on block 0 for the full duration. Bulk
loading N rows into an index with E existing elements was O(N·E) in
I/O under one lock. The reviewer re-verified on 2026-04-10 that the
issue was still present on HEAD and flagged it as the highest-value
unresolved late-10000s performance item.

## What Landed

### 1. Insert is now split into a first-insert path and a fast path

`src/am/insert.rs:9` now snapshots the metadata under a SHARE lock via
the existing `shared::read_metadata_page` helper before deciding what
to lock exclusively:

```rust
// Snapshot metadata under a SHARE lock so the duplicate scan does not
// serialize concurrent inserts behind the metadata-page exclusive lock.
let metadata_snapshot = shared::read_metadata_page(index_relation);

if metadata_snapshot.dimensions == 0 && metadata_snapshot.bits == 0 {
    // First-insert path: fall back to with_locked_metadata_page
    // (shape init atomicity matters, and the duplicate scan on an
    // effectively empty index is degenerate).
    ...
    return false;
}

// Fast path: shape is known, validate against the snapshot.
if tuple.dimensions != metadata_snapshot.dimensions
    || tuple.bits != metadata_snapshot.bits
    || tuple.seed != metadata_snapshot.seed
{
    pgrx::error!("tqhnsw aminsert requires matching tqvector shape ...");
}

// Duplicate scan runs with only SHARE locks on individual data pages.
if let Some(element_tid) = find_duplicate_element_tid(
    index_relation, heap_relation,
    metadata_snapshot.dimensions, metadata_snapshot.bits,
    tuple.gamma, code_len, &tuple.code,
) {
    coalesce_duplicate_heap_tid(index_relation, element_tid, code_len, heap_tid);
    return false;
}

let element_tid = append_heap_tuple(index_relation, &tuple);

// Only reacquire the metadata exclusive lock when the snapshot says
// entry_point still needs to be set. Re-check under the exclusive
// lock in case another insert beat us to it.
if metadata_snapshot.entry_point == page::ItemPointer::INVALID {
    shared::with_locked_metadata_page(index_relation, |metadata| {
        if metadata.entry_point == page::ItemPointer::INVALID {
            metadata.entry_point = element_tid;
        }
    });
}
```

Three things matter about this structure:

- **Shape validation is safe against the snapshot.** Shape fields
  (`dimensions`, `bits`, `seed`) are write-once: they go from zero to
  the first-insert values and never change. Once a snapshot sees
  non-zero shape, no concurrent writer can invalidate it.
- **`find_duplicate_element_tid` no longer sits inside the metadata
  exclusive lock.** It still takes a per-page `BUFFER_LOCK_SHARE` on
  each data page it scans, which is the only locking the scan
  actually needs. Concurrent inserts now run the duplicate scan in
  parallel.
- **`entry_point` update re-checks inside the exclusive lock.** The
  snapshot may say INVALID but another concurrent first-ish-insert
  may set it before us; the closure only writes when the re-check
  still sees INVALID, so we never clobber a racing initializer.

### 2. First-insert path is unchanged

When the snapshot shows `dimensions == 0 && bits == 0` we keep the
original `with_locked_metadata_page` closure verbatim: atomic shape
init + duplicate scan (degenerate on an empty index) + append +
`entry_point` set. This preserves the shape-init atomicity the
original code relied on, and the "O(pages) under exclusive lock"
complaint does not apply because there are no data pages yet.

### 3. Accepted race: concurrent same-code inserts may double-insert

The reviewer's request explicitly offered two options for the race
between step 2 (duplicate scan under SHARE) and step 3 (append under
exclusive data-page lock):

> Re-check after acquiring the exclusive lock, or accept the rare
> double-insert and let the next scan coalesce.

I took the second option. Re-checking would require either re-running
the full scan under an exclusive lock (which defeats the purpose) or
scanning only the pages modified since our snapshot (which introduces
version tracking we don't have). Accepting the race means that if two
concurrent inserts commit the same code in a narrow window, the index
ends up with two element tuples sharing a code instead of one tuple
with two heap_tids.

This is safe because:

- The query path already tolerates multiple element tuples with the
  same code — it compares codes and gamma and follows heap_tids from
  whichever tuple it reaches. Nothing indexes element tuples by code.
- The element tuples are semantically distinct — they have different
  `element_tid`s. A future coalescing pass or a subsequent duplicate
  insert could merge them, but correctness does not depend on that.
- The race is genuinely rare: it requires two backends to both not
  see each other's append during the duplicate scan, which means the
  two inserts must overlap within the scan window on a workload that
  actually inserts the same encoded vector twice. For the common
  bulk-insert case (many distinct vectors), the race cannot happen.

A comment at `src/am/insert.rs:104-109` documents this trade-off and
points back at the 10045 reviewer's explicit approval of it.

### 4. All other call sites of `with_locked_metadata_page` unchanged

`tqhnsw_aminsert` is the only caller of `with_locked_metadata_page`
outside of build/initialization code paths, so no other insert site
changes behavior.

## Evidence

### Validation matrix

```bash
cargo check --no-default-features --features pg17
cargo clippy --all-targets --no-default-features --features pg17,bench -- -D warnings
rustfmt --check src/am/insert.rs
cargo pgrx test pg17 test_tqhnsw_insert
cargo pgrx test pg17 test_tqhnsw_empty_index
```

All pass on this machine (Linux 6.17.0-19-generic, pgrx 0.17,
PostgreSQL 17.9 scratch cluster).

### Test output

`test_tqhnsw_insert` subset (9 tests, all aminsert-exercising):

```
test tests::pg_test_tqhnsw_insert_appends_new_element_tuple ... ok
test tests::pg_test_tqhnsw_insert_reuses_tail_page_when_space_remains ... ok
test tests::pg_test_tqhnsw_insert_allocates_new_page_when_tail_is_full ... ok
test tests::pg_test_tqhnsw_insert_reuses_new_tail_page_after_rollover ... ok
test tests::pg_test_tqhnsw_insert_coalesces_duplicate_vectors ... ok
test tests::pg_test_tqhnsw_insert_keeps_gamma_distinct ... ok
test tests::pg_test_tqhnsw_insert_rejects_duplicate_heaptid_overflow - should panic ... ok
test tests::pg_test_tqhnsw_insert_rejects_build_source_column_index - should panic ... ok
test tests::pg_test_tqhnsw_insert_rejects_mismatched_seed - should panic ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 243 filtered out
```

`test_tqhnsw_empty_index` subset (2 tests, first-insert path):

```
test tests::pg_test_tqhnsw_empty_index_insert_initializes_shape_metadata ... ok
test tests::pg_test_tqhnsw_empty_index_reuses_initialized_metadata ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 246 filtered out
```

Key coverage points these 11 tests exercise:

- First-insert shape init on an empty index
  (`empty_index_insert_initializes_shape_metadata`) — stays on the
  exclusive path.
- Second insert after shape init (`empty_index_reuses_initialized_metadata`)
  — exercises the new fast path.
- Tail-page reuse (`reuses_tail_page_when_space_remains`) — fast path
  with an existing data page.
- New-page rollover (`allocates_new_page_when_tail_is_full`,
  `reuses_new_tail_page_after_rollover`) — fast path with
  `append_heap_tuple_to_new_page` fallback.
- Duplicate coalescing (`coalesces_duplicate_vectors`) — fast path
  detects an existing element tuple and calls
  `coalesce_duplicate_heap_tid` without ever taking the metadata
  exclusive lock.
- Gamma-distinct same-code (`keeps_gamma_distinct`) — fast path, two
  element tuples.
- Overflow panic (`rejects_duplicate_heaptid_overflow`) — fast path
  coalesce with full heap_tid slots.
- Shape mismatch panic (`rejects_mismatched_seed`) — fast path
  shape validation against the snapshot.
- Source-column reject panic (`rejects_build_source_column_index`) —
  early reject before snapshot read.

### Formatting and lint

- `rustfmt --check src/am/insert.rs` clean.
- `make lint` clean (`cargo clippy --all-targets --no-default-features
  --features pg17,bench -- -D warnings` reports zero warnings).
- `make fmt-check` reports pre-existing drifts in other files
  (`src/am/mod.rs`, `src/lib.rs`, `src/quant/mse.rs`,
  `src/quant/prod.rs`, `tests/recall_integration.rs`) that also exist
  on untouched `main` — not introduced by this change and not in
  scope.

### Benchmarks — not re-run on this branch

I did not rerun a concurrency benchmark. The reviewer's original
feedback also did not rerun one, noting that the code path was
structurally unchanged. The commit is a structural lock-scope
narrowing: the hot path now holds no metadata lock across the
duplicate scan or the append, and the `entry_point` exclusive
re-lock only runs for the early inserts before `entry_point` is set.
The win shows up in any workload that drives multiple backends at the
index concurrently, and in the `O(N·E) → O(N·E / concurrency)`
reduction on bulk insert test-init time that the original packet
called out.

If you want quantitative numbers, the most direct next step is a
timed test-init run on a 1K-row fixture — that is where the original
packet reported the worst case.

## Why This Matters

This is the fix the 10045 reviewer explicitly flagged as "the
highest-value unresolved late-10000s performance item" and as a
prerequisite for "insert throughput or concurrent-insert numbers to
mean anything." It removes the `O(pages)`-under-exclusive-lock
serialization point from the hot insert path without changing the
insert semantics visible to callers and without rewriting the
duplicate scan itself.

## Files

- `src/am/insert.rs`
  - `tqhnsw_aminsert` restructured into a first-insert path and a
    fast path.
  - Shape validation moved onto the SHARE-read snapshot.
  - `find_duplicate_element_tid` now runs with no metadata lock held.
  - `entry_point` update reacquires the metadata exclusive lock only
    when the snapshot showed `INVALID`, with a re-check inside the
    lock.
  - New comments at lines 32-38, 86-88, 104-109, 125-127 document the
    split and the accepted race.

No other files changed. `shared::read_metadata_page`,
`shared::with_locked_metadata_page`, `find_duplicate_element_tid`,
`coalesce_duplicate_heap_tid`, and `append_heap_tuple` are all
unchanged.

## Out of Scope

- Duplicate-scan optimization itself (bloom filter, early exit,
  per-page tag index). The original 10045 packet called this out as
  a follow-up; narrowing the lock was the priority.
- Coalescing of duplicate element tuples created by the accepted
  race. Requires no new mechanism for query-path correctness.
- Any change to `ambuild` or the bulk build path.
- Any change to `with_locked_metadata_page` callers outside
  `tqhnsw_aminsert` (scan, debug readers, build init).
- Concurrency benchmarks or a timed test-init regression harness.
  If wanted, that is a separate follow-up.
