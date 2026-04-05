# Review: Build Holds All Vectors in Memory

**File:** `src/am/mod.rs:1233-1243` (`BuildState`)
**Severity:** Medium (memory pressure on large indexes)
**Category:** Resource management

## Finding

During `ambuild`, all heap tuples (including their full quantized code vectors) are accumulated in memory:

```rust
struct BuildState {
    heap_tuples: Vec<BuildTuple>,  // all tuples held in memory
    // ...
}
```

Each `BuildTuple` holds a `code: Vec<u8>` (772 bytes at 1536-dim 4-bit) plus heap TID vectors. For 1M rows, this is approximately:
- 772 bytes * 1M = ~736 MB just for codes
- Plus `Vec<ItemPointer>` overhead, source vectors if present, etc.

Additionally, `build_hnsw_graph` creates a second copy of all codes inside the `hnsw_rs::Hnsw` structure.

This means a 1M-row build with 1536-dim 4-bit vectors requires ~1.5GB+ of memory. Combined with `amusemaintenanceworkmem = true` (line 84), this could exceed `maintenance_work_mem` without any enforcement.

## Recommendation

1. **Short term:** Document the memory requirements in the README and/or as a WARNING during build if the estimated memory exceeds `maintenance_work_mem`.
2. **Medium term:** Consider a two-pass build strategy:
   - Pass 1: Build HNSW graph with codes only, discard full tuples
   - Pass 2: Re-scan heap to write pages
3. **Long term:** Streaming build with external sort for very large indexes.

Also: the `amusemaintenanceworkmem = true` flag tells the planner to use `maintenance_work_mem` for builds, but the code never actually checks or respects this limit.

## Action Required

At minimum, log an INFO/NOTICE message during build reporting estimated memory usage. Consider checking `maintenance_work_mem` and erroring if the expected memory significantly exceeds it.
