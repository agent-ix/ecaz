# Review Request: C1 ADR-030 V2 PqFastScan Vacuum On Built Indexes

## Context

Packet 382 added live insert for non-empty built `PqFastScan` indexes, but
vacuum still hard-rejected grouped storage:

- `amvacuumcleanup` could not count grouped live nodes
- pass 1 only knew how to strip dead heap TIDs from scalar element tuples
- pass 2 repair request discovery only knew how to walk scalar element tuples
- pass 3 finalization only knew how to tombstone scalar element tuples

That left built `PqFastScan` indexes unable to tolerate heap deletes, even
though exact graph reads and live insert already ran through
`GraphStorageDescriptor`.

## Problem

Before this packet, grouped vacuum was blocked at the top level with:

- `tqhnsw vacuum does not support PqFastScan indexes yet`

That reject was now the wrong architecture boundary.

For built `PqFastScan` indexes, vacuum does not need a separate algorithm. It
needs the same three phases as `TurboQuant`:

1. strip dead heap TIDs from the live node payload
2. repair/unlink stale graph refs
3. finalize fully dead nodes to `deleted = true`

The real missing piece was storage-aware tuple decoding and rewrite, not a
second vacuum lifecycle.

## Planned Slice

One vacuum checkpoint:

1. remove the top-level `PqFastScan` vacuum reject
2. count grouped hot tuples in `amvacuumcleanup`
3. make vacuum pass 1 rewrite grouped hot tuples
4. make repair-request discovery and finalization storage-aware
5. add pg coverage for grouped stats, duplicate compaction, and dead-edge
   unlink/finalize

Empty-index bootstrap and additional replacement-top-up work remain out of
scope for this packet.

## Implementation

Updated:

- `src/am/shared.rs`
- `src/am/vacuum.rs`
- `src/lib.rs`

### 1. Vacuum stats now count live grouped hot tuples

`shared::count_element_tuples(...)` now resolves
`GraphStorageDescriptor::from_metadata(...)` and counts:

- live `TQ_ELEMENT_TAG` tuples for `TurboQuant`
- live `TQ_GROUPED_HOT_TAG` tuples for `PqFastScan`

That lets `amvacuumcleanup` reuse the same noop stats path for both formats.

### 2. Pass 1 now rewrites grouped hot tuples

`run_bulkdelete_with_adapter(...)` now drives pass 1 through a storage
descriptor rather than a scalar code length.

`plan_page_pass1(...)` and `apply_page_pass1_updates(...)` now handle:

- `TqElementTuple` for `TurboQuant`
- `TqGroupedHotTuple` for `PqFastScan`

For grouped storage, pass 1 now:

- strips dead heap TIDs from grouped hot tuples
- records grouped hot tuples with no surviving heap TIDs for finalization
- preserves the rest of the grouped hot payload layout in place

### 3. Repair request discovery now walks grouped storage too

Vacuum repair is now dispatched through:

- `repair_graph_connections_with_storage(...)`

`collect_repair_requests_on_page(...)` now derives
`(level, deleted, heaptids_empty, neighbortid)` from either:

- `TqElementTuple`
- `TqGroupedHotTuple`

That means pass 2 now discovers broken grouped graph edges the same way it
already did for scalar storage, while still reusing shared neighbor unlink and
replacement planning.

### 4. Finalization is now storage-aware

The old scalar-only finalization path is replaced with:

- `finalize_fully_dead_elements_with_storage(...)`

That now tombstones:

- scalar element tuples for `TurboQuant`
- grouped hot tuples for `PqFastScan`

In both cases the finalize step only flips `deleted = true` after pass 1 has
already cleared the last surviving heap TID.

### 5. Added grouped pg coverage

`src/lib.rs` now covers:

- `debug_vacuum_stats(...)` on a built `PqFastScan` index
- grouped duplicate heap-TID compaction in pass 1
- grouped dead-edge unlink plus grouped-hot finalization in pass 2/3

The old grouped vacuum rejection test is removed because that reject no longer
exists.

## Measurements

No new benchmark or recall measurements in this slice. This is vacuum parity
groundwork only.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`,
  `CopyErrorData`, and `errstart`

## Outcome

This checkpoint removes the last blanket grouped-vacuum reject for built
indexes:

1. `PqFastScan` vacuum stats now count live grouped hot tuples
2. pass 1 strips dead heap TIDs from grouped hot tuples
3. pass 2 can discover and repair grouped dead-edge damage
4. pass 3 finalizes fully dead grouped hot tuples

What it still does **not** do:

- first-insert bootstrap for empty `PqFastScan` indexes
- grouped linear top-up for repair replacement search when graph search yields
  too few replacements
- broader reloption / naming cleanup from ADR-032 task 15

## Next Slice

The next practical slices are:

1. remove the remaining experimental build gating in favor of first-class
   `storage_format` selection
2. decide whether grouped repair replacement search needs a storage-aware
   fallback/top-up path beyond graph-search-only candidates
3. finish insert/vacuum parity details needed to land the coexisting-format
   contract from ADR-032 on `main`
