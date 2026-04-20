# Ecaz PG18 Upgrade & Optimization Plan

## Overview

PostgreSQL 18 introduces async I/O, new index AM callbacks, custom statistics/EXPLAIN
APIs, and GIN parallel build infrastructure. This document maps each PG18 capability to
concrete changes in tqvector, ordered by impact.

---

## 1. Async I/O via `read_stream` — Graph Scan Prefetch

**Impact: HIGH — 3-4x cold-cache speedup on graph traversal**

### Background

PG18 introduces `ReadStream` — an adaptive prefetch pipeline that sits on top of the
new AIO subsystem (`io_method=sync|worker|io_uring`). Extensions call a single API and
get transparent async I/O regardless of the backend method.

Thomas Munro prototyped this on pgvector's HNSW and measured **4x speedup** on cold
cache with `effective_io_concurrency=12-16`.

### API

```c
// Callback: return next block to prefetch, or InvalidBlockNumber when done
typedef BlockNumber (*ReadStreamBlockNumberCB)(
    ReadStream *stream, void *callback_private_data, void *per_buffer_data);

// Create stream
ReadStream *read_stream_begin_relation(
    int flags,                        // READ_STREAM_DEFAULT for index scans
    BufferAccessStrategy strategy,    // NULL for index scans
    Relation rel,
    ForkNumber forknum,
    ReadStreamBlockNumberCB callback,
    void *callback_private_data,
    size_t per_buffer_data_size);

// Consume next prefetched buffer (pinned, must ReleaseBuffer)
Buffer read_stream_next_buffer(ReadStream *stream, void **per_buffer_data);

// Reset for reuse (NOTE: resets adaptive distance to 1)
void read_stream_reset(ReadStream *stream);

// Cleanup
void read_stream_end(ReadStream *stream);
```

### Changes Required

#### A. Graph neighbor prefetch (`scan.rs` / `graph.rs`)

The hottest I/O path is `refill_candidate_frontier_from_source()` (scan.rs:677-717).
When we expand a node, we load its neighbor list, then load each neighbor element
one-at-a-time via `ReadBufferExtended`. With `m=16`, that's up to 16 synchronous
page reads per expansion.

**New pattern:**

```
1. load_graph_adjacency(source_tid)       → get neighbor TID list
2. Collect neighbor block numbers into callback state
3. read_stream_reset(stream)              → feed neighbor blocks
4. for each read_stream_next_buffer():
     - decode element tuple from prefetched buffer
     - score against query
     - seed into frontier if promising
5. ReleaseBuffer each
```

Key design decisions:
- Create the `ReadStream` once in `amrescan`, store in `TqScanOpaque`, destroy in `amendscan`
- The callback returns block numbers from a `Vec<BlockNumber>` populated before each
  `read_stream_reset()` call
- Use `READ_STREAM_DEFAULT` flag (respects `effective_io_concurrency` GUC)
- No `BufferAccessStrategy` (index scans shouldn't use ring buffers)

**Gotcha — `read_stream_reset()` forgets readahead distance.** Munro identified this as
problematic for HNSW where neighbor batches come in bursts. Check if the
`reset_distance` patch landed in PG18 release. If not, consider keeping the stream
alive across expansions and feeding blocks incrementally rather than resetting.

#### B. Linear scan prefetch (`scan.rs:832-906`)

`next_linear_scan_heap_tid()` walks blocks sequentially — ideal for streaming I/O.

**New pattern:**

```
1. Create a linear-scan ReadStream with READ_STREAM_SEQUENTIAL flag
2. Callback simply returns next_block_number++
3. Replace the for-loop ReadBufferExtended with read_stream_next_buffer()
4. io_combine_limit will merge consecutive blocks into single I/O ops
```

This is simpler than the graph case because blocks are strictly sequential.
`io_combine_limit` (default 16 blocks = 128KB) will batch them automatically.

#### C. `count_element_tuples()` in vacuum (`shared.rs:142-188`)

Same sequential pattern as the linear scan. Convert to streaming reads.

#### D. PG version compatibility

```rust
#[cfg(feature = "pg18")]
mod read_stream_scan { /* streaming implementation */ }

#[cfg(not(feature = "pg18"))]
mod legacy_scan { /* current ReadBufferExtended implementation */ }
```

For PG16/17, optionally add `PrefetchBuffer()` calls as a lighter-weight improvement
(OS-level readahead hints only, no true async).

---

## 2. Cost Estimation — Actually Enable the Planner

**Impact: HIGH — currently the index is unusable by the planner**

### Current State

`cost.rs` sets `index_startup_cost = f64::MAX` and `index_total_cost = f64::MAX`,
which means the planner will **never** choose this index. This was intentional during
development but needs to be fixed for production use.

### Changes Required

Implement a real cost model in `ec_hnsw_amcostestimate`:

```rust
// Estimated I/O for bootstrap phase:
// - Entry point: 2 page reads (element + neighbors)
// - Per expansion: ~m page reads for neighbors, ~m for elements
// - Total bootstrap: ~ef_search * 2m page reads
let bootstrap_pages = ef_search as f64 * 2.0 * m as f64;

// Linear scan fallback: reads all remaining pages
let total_pages = index_pages;
let linear_pages = total_pages - bootstrap_pages.min(total_pages);

// CPU cost: scoring each element
let per_tuple_cpu = cpu_operator_cost * dimensions as f64;

*index_startup_cost = bootstrap_pages * seq_page_cost;
*index_total_cost = bootstrap_pages * random_page_cost
                  + linear_pages * seq_page_cost
                  + num_index_tuples * per_tuple_cpu;
*index_selectivity = 1.0;  // ORDER BY returns all rows
*index_correlation = 0.0;  // no correlation with heap order
*index_pages = total_pages;
```

### PG18 bonus: `amgettreeheight`

New callback lets the planner cache the index structure height. For HNSW, return
`max_level` from the metadata page:

```rust
// routine.rs — add to IndexAmRoutine:
amroutine.amgettreeheight = Some(ec_hnsw_amgettreeheight);

// Implementation:
unsafe extern "C-unwind" fn ec_hnsw_amgettreeheight(rel: pg_sys::Relation) -> i32 {
    let metadata = shared::read_metadata_page(rel);
    metadata.max_level as i32
}
```

The planner stores this in `IndexOptInfo.tree_height` and passes it to
`amcostestimate` via `IndexPath.indexinfo`. Use it to refine the bootstrap
cost estimate (higher layers = fewer hops to reach the neighborhood).

---

## 3. Parallel Index Build

**Impact: MEDIUM-HIGH — build time scales linearly with data today**

### Current State

- `amcanbuildparallel = false` (`src/am/ec_hnsw/routine.rs`)
- `build_hnsw_graph()` is still single-threaded (`src/am/ec_hnsw/build.rs`)
- The native `ec_hnsw` builder still runs leader-only after heap-tuple collection
- Heap scan uses `table_index_build_scan` without parallel flags

### GIN Parallel Build Pattern (PG18)

GIN uses this architecture, which we can adapt:

```
Leader:
  1. CreateParallelContext("postgres", "_ec_hnsw_parallel_build_main", nworkers)
  2. Allocate shared memory: TqBuildShared + Sharedsort + WalUsage + BufferUsage
  3. InitializeParallelDSM → LaunchParallelWorkers
  4. Leader participates in parallel heap scan too
  5. Leader waits via ConditionVariable for workers
  6. Leader reads sorted results from Sharedsort → builds HNSW graph

Workers:
  1. Open heap/index via shm_toc lookup
  2. Parallel heap scan → encode tqvectors → write (heap_tid, code) to Sharedsort
  3. Signal completion via ConditionVariable
```

### Phase 1: Parallel Heap Scan + Encoding

The heap scan and tqvector encoding are embarrassingly parallel. Each worker:
- Scans a portion of the heap via `ParallelTableScanDesc`
- Detoasts and validates each tqvector datum
- Writes `(gamma, code, heap_tids)` tuples into a shared sort

This parallelizes the I/O-bound heap scan and CPU-bound detoast/validation.

### Phase 2: Native Graph Construction (future)

The current native builder is still leader-only. Options:
1. **Keep serial initially** — leader builds graph from sorted tuples (matches GIN pattern)
2. **Parallelize the native builder** — shard candidate discovery / layer work inside Ecaz's own builder
3. **Batch parallel** — partition vectors, build sub-graphs in parallel, merge

Recommendation: Start with Phase 1 (parallel scan + encode), keep graph build serial.
This already parallelizes the I/O bottleneck.

### Changes Required

```rust
// routine.rs
amroutine.amcanbuildparallel = true;

// build.rs — new parallel infrastructure
// - TqBuildShared struct (shared state)
// - _ec_hnsw_parallel_build_main (worker entry point)
// - Parallel heap scan via table_beginscan_parallel
// - Sharedsort for coordinated tuple collection
```

---

## 4. Vacuum — Actually Delete Things

**Impact: MEDIUM — currently vacuum is a no-op**

### Current State

`ambulkdelete` and `amvacuumcleanup` both call `ec_hnsw_noop_vacuum_stats` which
just counts tuples. Dead tuples are never removed from the index.

### Changes Required

1. **Soft delete**: Mark `element.deleted = true` for dead heap TIDs
2. **Heap TID removal**: Remove specific heap TIDs from element's `heaptids` array
3. **Graph maintenance**: When an element has zero remaining heap TIDs, mark deleted
   and optionally rewire its neighbors
4. **Page compaction**: Optional — compact pages with many deleted tuples

The HNSW graph tolerates soft-deleted nodes (scan already checks `element.deleted`),
so full graph rewiring isn't strictly required for correctness.

### PG18 bonus: streaming reads for vacuum scan

Use `read_stream` to scan index pages during vacuum, same pattern as linear scan.

---

## 5. Strategy Translation — `amtranslatestrategy` / `amtranslatecmptype`

**Impact: LOW-MEDIUM — enables optimizer features**

### Background

PG18 adds `CompareType` as a generic comparison abstraction. The AM translates between
its private strategy numbers and `CompareType` values.

```c
typedef enum CompareType {
    COMPARE_INVALID = 0,
    COMPARE_LT, COMPARE_LE, COMPARE_EQ, COMPARE_GE, COMPARE_GT, COMPARE_NE,
    COMPARE_OVERLAP, COMPARE_CONTAINED_BY,
} CompareType;
```

### Changes Required

tqvector uses strategy 1 for the `<#>` operator (ORDER BY negative inner product).
This doesn't map to a standard comparison type, so:

```rust
// routine.rs
amroutine.amtranslatestrategy = Some(ec_hnsw_amtranslatestrategy);
amroutine.amtranslatecmptype = Some(ec_hnsw_amtranslatecmptype);

unsafe extern "C-unwind" fn ec_hnsw_amtranslatestrategy(
    strategy: pg_sys::StrategyNumber,
    _opfamily: pg_sys::Oid,
) -> pg_sys::CompareType {
    match strategy {
        1 => pg_sys::CompareType::COMPARE_LT,  // ORDER BY ASC = closest first
        _ => pg_sys::CompareType::COMPARE_INVALID,
    }
}

unsafe extern "C-unwind" fn ec_hnsw_amtranslatecmptype(
    cmptype: pg_sys::CompareType,
    _opfamily: pg_sys::Oid,
) -> pg_sys::StrategyNumber {
    match cmptype {
        pg_sys::CompareType::COMPARE_LT => 1,
        _ => pg_sys::InvalidStrategy,
    }
}
```

Also set the new boolean flags:
```rust
amroutine.amconsistentequality = false;
amroutine.amconsistentordering = true;  // we do ORDER BY
```

---

## 6. Custom EXPLAIN Options

**Impact: MEDIUM — essential for debugging and tuning**

### PG18 API

```c
RegisterExtensionExplainOption("ecaz", handler, GUCCheckBooleanExplainOption);
explain_per_node_hook = ec_hnsw_explain_hook;
```

### What to expose

`EXPLAIN (ecaz) SELECT ... ORDER BY embedding <#> query LIMIT 10`:

```
Index Scan using idx_embedding on items
  Order By: (embedding <#> '{...}'::real[])
  Ecaz Stats:
    Bootstrap candidates expanded: 3
    Bootstrap pages read: 47
    Linear scan pages read: 0
    Elements scored: 156
    Heap TIDs returned: 10
    Quantizer cache hit: true
```

### Changes Required

1. Add counters to `TqScanOpaque`:
   - `bootstrap_pages_read`, `linear_pages_read`
   - `elements_scored`, `elements_skipped_deleted`
   - `graph_expansions`

2. Register the EXPLAIN option in `_PG_init` (need to add a `_PG_init` function)

3. Implement the `explain_per_node_hook` to emit stats when the option is enabled

---

## 7. Custom Cumulative Statistics

**Impact: LOW-MEDIUM — operational visibility**

### PG18 API

Register a custom pgstat kind to track aggregate metrics across all queries:

```rust
// In _PG_init:
pgstat_register_kind(PGSTAT_KIND_EXPERIMENTAL, &ec_hnsw_kind_info);
```

### What to track

- Total distance calculations
- Total graph hops
- Total linear scan pages
- Bootstrap hit rate (queries satisfied entirely by graph traversal)
- Quantizer cache hit/miss

These would be visible via an `ecaz_stats()` SQL function or a custom view.

### Priority

Lower priority than EXPLAIN options. Implement after the core I/O and planner
improvements are stable.

---

## 8. `PG_MODULE_MAGIC_EXT`

**Impact: LOW — easy win for diagnostics**

```rust
// In pgrx, this may need a raw extern block or pgrx macro support.
// Check pgrx 0.18+ for PG18 support.
// The extension name/version become visible via pg_get_loaded_modules().
```

---

## 9. pgrx PG18 Support

### Current State

- `pgrx = "0.17"` supports PG14-17
- PG18 support requires pgrx 0.18+ (check release status)

### Changes Required

```toml
[features]
default = ["pg18"]   # change default
pg14 = ["pgrx/pg14", "pgrx-tests/pg14"]
pg15 = ["pgrx/pg15", "pgrx-tests/pg15"]
pg16 = ["pgrx/pg16", "pgrx-tests/pg16"]
pg17 = ["pgrx/pg17", "pgrx-tests/pg17"]
pg18 = ["pgrx/pg18", "pgrx-tests/pg18"]

[dependencies]
pgrx = "0.18"  # or whatever version adds pg18
```

### New `IndexAmRoutine` fields in PG18

pgrx bindings should expose these new fields automatically:
- `amgettreeheight`
- `amtranslatestrategy`
- `amtranslatecmptype`
- `amconsistentequality`
- `amconsistentordering`

If pgrx doesn't expose them yet, use raw `pg_sys` field access.

---

## Implementation Order

| Phase | Work Item | Files | Effort |
|-------|-----------|-------|--------|
| 0 | pgrx PG18 upgrade | Cargo.toml | 1-2d |
| 1a | Cost estimation (unblock planner) | cost.rs | 1d |
| 1b | `amgettreeheight` | routine.rs, shared.rs | 2h |
| 2a | Linear scan `read_stream` | scan.rs | 1-2d |
| 2b | Graph neighbor `read_stream` prefetch | scan.rs, graph.rs | 2-3d |
| 2c | Vacuum scan `read_stream` | shared.rs | 4h |
| 3 | Strategy translation callbacks | routine.rs | 2h |
| 4 | EXPLAIN stats counters | scan.rs, lib.rs | 1-2d |
| 5 | Vacuum soft-delete | vacuum.rs, page.rs | 2-3d |
| 6 | Parallel heap scan for build | build.rs | 3-5d |
| 7 | Custom pgstat metrics | lib.rs (new) | 1-2d |
| 8 | PG_MODULE_MAGIC_EXT | lib.rs | 1h |

### Critical Path

```
Phase 0 (pgrx upgrade)
  ↓
Phase 1a+1b (cost model — unblocks all query testing)
  ↓
Phase 2a (linear scan streaming — easiest read_stream integration)
  ↓
Phase 2b (graph prefetch — highest performance impact)
  ↓
Phase 3-8 (independent, can parallelize)
```

---

## Benchmarking Plan

### Cold Cache Scan (primary metric for AIO)

```sql
-- Evict buffers (PG18)
SELECT pg_buffercache_evict_relation('idx_embedding'::regclass);

-- Time a scan
EXPLAIN (ANALYZE, BUFFERS, WAL)
SELECT id FROM items ORDER BY embedding <#> $query LIMIT 10;
```

Compare with `io_method=sync` vs `io_method=worker` vs `io_method=io_uring`.
Vary `effective_io_concurrency` from 1 to 32.

### Build Performance

```sql
-- Parallel build
SET max_parallel_maintenance_workers = 4;
CREATE INDEX CONCURRENTLY ON items USING ec_hnsw (embedding);
```

### Key GUCs for tuning

```
effective_io_concurrency = 16      -- async I/O depth for scans
maintenance_io_concurrency = 16    -- async I/O depth for builds/vacuum
io_combine_limit = 16              -- max pages per I/O op (128KB default)
io_method = io_uring               -- best performance on Linux 5.1+
```

---

## Architecture Considerations

### read_stream lifetime management

The `ReadStream` should be created once per scan (in `amrescan`) and destroyed in
`amendscan`. Between graph expansions, use `read_stream_reset()` to repopulate the
callback's block list. This avoids repeated stream creation overhead.

Store in `TqScanOpaque`:
```rust
pub(super) graph_read_stream: *mut pg_sys::ReadStream,
pub(super) linear_read_stream: *mut pg_sys::ReadStream,
pub(super) prefetch_blocks: Vec<u32>,   // callback feeds from this
pub(super) prefetch_index: usize,       // current position in prefetch_blocks
```

### Two streams vs one

Use **two separate streams**:
1. **Graph stream** — random access, `READ_STREAM_DEFAULT`, used during bootstrap
2. **Linear stream** — sequential access, `READ_STREAM_SEQUENTIAL`, used in fallback

They serve different I/O patterns and shouldn't share adaptive distance state.

### Callback design

```rust
// Graph prefetch callback
unsafe extern "C" fn graph_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    cb_data: *mut c_void,
    _per_buffer: *mut c_void,
) -> pg_sys::BlockNumber {
    let state = &mut *(cb_data as *mut PrefetchState);
    if state.index >= state.blocks.len() {
        return pg_sys::InvalidBlockNumber;
    }
    let block = state.blocks[state.index];
    state.index += 1;
    block
}
```
