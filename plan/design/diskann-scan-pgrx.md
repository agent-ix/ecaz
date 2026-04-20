# DiskANN (Vamana) Scan — pgrx wiring (Phase 6B)

Design doc for the pgrx callback layer that binds Phase 6A's pure-
Rust scan shell (`src/am/diskann/scan.rs::vamana_scan_with`) to
Postgres's index-scan protocol. This doc crystallizes the Phase 6B
work so it drops in cleanly when the native-build lane merges.

- **Target modules.** `src/am/diskann/routine.rs` (stub replacement),
  a new `src/am/diskann/scan_state.rs` (the opaque cross-callback
  state), and possibly a thin `src/am/diskann/scan_callbacks.rs` if
  `scan_state` grows past ~400 lines.
- **Does not touch.** `src/am/scan.rs`, `src/am/routine.rs`,
  `src/am/scan_debug.rs` (tqhnsw side). Read them freely for
  reference; do not edit until the native-build merge.
- **Preconditions.** Phase 5C-3 (`ambuild` + quantizer training)
  and Phase 5D (persisted-graph reader) have landed on `main`; the
  metadata page schema is the one Phase 5C-2 writes.

## Goals

- Wire `ambeginscan` / `amrescan` / `amgettuple` / `amendscan` for
  `ecdiskann` on top of `vamana_scan_with`.
- Keep scan-level state in a single `DiskannScanOpaque` allocated in
  the scan's memory context; do not leak allocations between
  cursor iterations.
- Allocation-free per-`amgettuple` hot path: the greedy frontier
  `VisitedState` is owned by the opaque and reused via `clear()`
  per rescan, not per gettuple.
- Bind `prefilter` to the Phase 1 quantizer scorer (`prepare_scorer`
  on the loaded `Quantizer`) and `rerank` to
  `ecvector::exact_distance` (heap cold path).

## Non-goals

- Parallel scan (`amcanparallel = false` already).
- Bitmap scan (`amgetbitmap = None` already).
- Backward cursor (`amcanbackward = false` already).
- Custom cost model. Phase 6B keeps the Phase 1A
  `disable_cost` surface until a Phase 6C cost pass.
- MVCC snapshot propagation into the scan shell. The rerank
  closure reads the heap through the scan's `EState` snapshot; the
  shell stays snapshot-agnostic.

## State layout

```rust
// src/am/diskann/scan_state.rs
pub(super) struct DiskannScanOpaque {
    // --- quantizer + query decode ---
    pub(super) query_dimensions: u16,
    pub(super) query_values: *mut f32,             // owned, palloc'd
    pub(super) prepared_scorer: *mut PreparedScorer, // from Quantizer::prepare_scorer
    pub(super) cached_quantizer: *const Quantizer,

    // --- graph identity ---
    pub(super) metadata: DiskannMetadata, // copied out of meta page at beginscan
    pub(super) chain: DataPageChain,      // constructed once at beginscan

    // --- scan-level scratch ---
    pub(super) visited: VisitedState,     // reused across rescan via clear()
    pub(super) result_buf: Vec<ScanResult>,
    pub(super) result_cursor: usize,

    // --- rerank wiring ---
    pub(super) heap_relation: pg_sys::Relation,     // shared with scan desc
    pub(super) heap_snapshot: pg_sys::Snapshot,
    pub(super) rerank_slot: *mut pg_sys::TupleTableSlot,

    // --- scan params (immutable after amrescan) ---
    pub(super) list_size: usize,          // from reloption or GUC
    pub(super) rerank_budget: usize,
    pub(super) top_k: usize,              // tracked across gettuple
    pub(super) rescan_called: bool,
}
```

The opaque pointer is installed into
`IndexScanDesc.opaque`; `ambeginscan` allocates in
`CurrentMemoryContext` (the scan's); `amendscan` reverses in order
(payload free → prepared_scorer free → opaque free).

## Callback sequencing

### `ecdiskann_ambeginscan`

1. Allocate `DiskannScanOpaque` zeroed into `CurrentMemoryContext`.
2. Buffer-pin the metadata page (block 0) under
   `BUFFER_LOCK_SHARE`, copy the `DiskannMetadata` struct out,
   release the lock (pin can drop — we have a copy).
3. Construct `DataPageChain` from the metadata's chain head block.
4. Load the cached `Quantizer` (Phase 1 trait). Same pattern as
   `src/am/scan.rs` tqhnsw side.
5. Allocate `VisitedState::new()` (single allocation; reused via
   `clear` on each rescan).
6. Set `opaque.rescan_called = false`.
7. Wire `scan.opaque = Box::into_raw(opaque) as *mut c_void`.

No query decode happens here — the ORDER BY operand is bound in
`amrescan`.

### `ecdiskann_amrescan`

1. Retrieve the opaque from `scan.opaque`.
2. If `rescan_called`, free the previous `prepared_scorer` and
   `query_values` in the opaque's context; `opaque.visited.clear()`.
3. Decode the ORDER BY key into `opaque.query_values` (f32 slice;
   same decode path as tqhnsw). Validate dimensions against the
   metadata's declared dimensions — error if mismatch (inherits the
   tqhnsw pattern).
4. Call `opaque.cached_quantizer.prepare_scorer(query_values)`
   into `opaque.prepared_scorer`.
5. Derive `list_size`, `rerank_budget`, `top_k` from reloptions +
   `LIMIT` if supplied. Keep defaults conservative
   (`L = 100`, `rerank_budget = 64`, `top_k = 10`) matching Phase
   5A's build-side baseline.
6. Run the scan eagerly:

   ```rust
   let reader = PersistedGraphReader {
       chain: &opaque.chain,
       graph_degree_r: opaque.metadata.graph_degree_r,
       binary_word_count: opaque.metadata.binary_word_count as usize,
       search_code_len: opaque.metadata.search_code_len as usize,
   };

   let prefilter = |t: &VamanaNodeTuple|
       unsafe { prepared_scorer_score(opaque.prepared_scorer, t) };

   let rerank = |hip: ItemPointer|
       unsafe { heap_rerank_exact(opaque, hip) };

   let params = ScanParams {
       entry_point: opaque.metadata.medoid_tid,
       list_size: opaque.list_size,
       rerank_budget: opaque.rerank_budget,
       top_k: opaque.top_k,
   };

   opaque.result_buf = vamana_scan_with(
       &reader,
       &mut opaque.visited,
       params,
       prefilter,
       rerank,
   ).unwrap_or_else(|e| pgrx::error!("ecdiskann scan: {e}"));
   opaque.result_cursor = 0;
   opaque.rescan_called = true;
   ```

Eager-materialize trades a one-shot compute burst for the
simplest possible `amgettuple`. For `top_k` in the tens-to-low-
hundreds this is cheaper than a pull-driven iterator (no Rust-side
coroutine machinery, no borrow lifetime through `scan.opaque`).

### `ecdiskann_amgettuple`

```rust
unsafe extern "C-unwind" fn ecdiskann_amgettuple(
    scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    let opaque = /* cast scan.opaque */;
    if opaque.result_cursor >= opaque.result_buf.len() {
        return false;
    }
    let hit = opaque.result_buf[opaque.result_cursor];
    opaque.result_cursor += 1;
    (*scan).xs_heaptid = hit.primary_heaptid.into_pg();
    (*scan).xs_recheckorderby = false;
    true
}
```

No work happens here beyond the cursor tick. The design
deliberately keeps this callback cheap and allocation-free so
PostgreSQL can call it in the inner tuple loop without surprise
latency spikes.

**MVCC filtering.** `xs_heaptid` is returned unfiltered; the
executor's heap fetch applies the snapshot visibility check.
Tombstoned index entries (Phase 8's `deleted = true`) are
filtered at the scan-shell layer by the rerank closure returning
`f32::INFINITY` for tombstoned tuples. The shell's sort drops
them to the tail; top-k truncation removes them.

### `ecdiskann_amendscan`

1. Free `prepared_scorer` if non-null.
2. Free `query_values` if non-null.
3. Drop the `VisitedState` (its `HashSet<ItemPointer>`s deallocate
   their backing storage).
4. Drop `result_buf`.
5. `Box::from_raw(opaque)` → drop.

## Locking

- Metadata page: `BUFFER_LOCK_SHARE` taken briefly in
  `ambeginscan`, released before the scan runs. The copied
  `DiskannMetadata` is used for the lifetime of the scan.
- Data-chain pages: inherit `DataPageChain`'s own buffer discipline
  inside `PersistedGraphReader::decode_tuple`. Phase 5D's reader
  already pins each page under `BUFFER_LOCK_SHARE` for the
  duration of a single tuple decode.
- Heap: rerank closure takes a share lock on the heap tuple it
  reads. This is the standard executor pattern — no change.

No cross-callback lock is held. The buffer pin / page lock
discipline is entirely inside Phase 5D's reader.

## Quantizer binding

The Phase 1 `Quantizer` trait exposes `prepare_scorer(query) ->
PreparedScorer`. `PreparedScorer` is an opaque state object that
binds a query vector + precomputed tables for the prefilter
kernel. For ecdiskann:

- Grouped PQ4 scorer uses `binary_words` and `search_code` from
  the tuple (Phase 5B `VamanaNodeTuple`) — both slices live on
  the decoded tuple.
- Binary Hamming prefilter (if elected by reloption) uses just
  `binary_words`.

Exact rerank (`rerank` closure) calls
`ecvector::exact_distance(query_values, heap_vec)` after a heap
fetch. No caching — rerank is bounded by `rerank_budget`.

## Error handling

All errors inside the shell are `Result<_, String>`. The pgrx
boundary converts them with `pgrx::error!(...)` under the
`pgrx_extern_c_guard`. No `panic!` should cross FFI.

## Concurrency

- **Scan + concurrent insert** (ADR-046): the reader takes share
  locks per-page; insert holds exclusive briefly per ADR-046. The
  reader's `decode_tuple` will see either the pre- or post-insert
  state; stale tuple reads are tolerated since they don't
  dereference free pointers (ADR-045 Decision 3 fixed-length).
- **Scan + concurrent vacuum** (ADR-047): fill-only vacuum writes
  (`repair_neighbors`, `mark_deleted`) don't shift tuple offsets
  thanks to ADR-045 Decision 3. A scan that reads a tuple
  mid-vacuum may see a stale neighbor list — acceptable since
  vacuum's fill-only posture never evicts live neighbors.

No ordered-lock-pass invariant applies to scans: they are read-only
and never hold more than one data-page lock.

## Open questions (non-blocking)

- **Reloption surface.** `list_size`, `rerank_budget` should be
  reloptions on the index. Defaults above are fine for first land.
  Finalize the option names with the tqhnsw reloption convention.
- **top_k source.** `LIMIT N` may not be visible to `amrescan`.
  First land can default to `rerank_budget` and let the executor
  truncate externally; a later pass can plumb `IndexScanState::
  ss.ps.plan->plan_rows` if available.
- **Fallback entry point.** If the medoid TID points to a deleted
  element (ADR-047 G2), the scan should fall back to a live TID.
  Simplest: if `decode_tuple(medoid).deleted`, iterate the chain
  in block order to find the first live TID. Defer to post-merge
  when ADR-047 G2 resolves.

## Out of scope (document pointers)

- Cost model — Phase 6C.
- EXPLAIN counters — mirror tqhnsw's `TqExplainCounters` shape.
- Parallel scan — out of scope per `amcanparallel = false`.
- Prefetch — `LinearPrefetchState`-style block prefetch can be
  added post-merge once flamegraphs show it pays.

## Test plan (for Phase 6B once it lands)

- Unit tests inside `scan_state.rs` that exercise the opaque's
  state machine with a fake reader (the same chain fixtures Phase
  6A uses).
- pgrx integration test (`sql/tests/ecdiskann_*.sql`) covering
  CREATE INDEX → rescan → top-10 → rescan-with-new-query →
  endscan. Reuse the tqhnsw test harness's fixture loader.
- Concurrent scan + insert smoke test once Phase 7 lands.

## References

- Phase 5D — `review/11022-phase5d-persisted-graph-reader/`
- Phase 6A — `review/11023-phase6a-scan-algorithm-shell/`
- VisitedState refactor — `review/11026-visited-state-reuse/`
- ADR-045 — `spec/adr/ADR-045-page-layout-discipline-for-graph-access-methods.md`
- ADR-046 — `spec/adr/ADR-046-vamana-insert-lock-ordering.md` (PROPOSED)
- ADR-047 — `spec/adr/ADR-047-vamana-vacuum-lock-ordering.md` (PROPOSED)
- Prior art: `src/am/scan.rs` (tqhnsw) — reference for reloption
  decode, query decode, and cached quantizer lookup patterns.
