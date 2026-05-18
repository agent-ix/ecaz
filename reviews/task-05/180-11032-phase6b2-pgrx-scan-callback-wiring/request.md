# Review Request: pgrx Scan Callback Wiring (Phase 6B-2)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/routine.rs`,
`src/am/ec_diskann/scan_state.rs` (new),
`src/am/ec_diskann/options.rs`,
`src/am/ec_diskann/mod.rs`,
`src/am/ec_diskann/ambuild.rs`

## What this packet is

Phase 6B-2 wires the `ec_diskann` ordered-scan callbacks through the
pure-Rust scan shell from Packet 11031.

The slice adds:

1. **`materialize_chain_from_index(index_relation)`** — copies block 0's
   `VamanaMetadataPage` under `BUFFER_LOCK_SHARE`, validates
   `INDEX_FORMAT_V3_DISKANN`, then walks blocks `1..RelationGetNumberOfBlocksInFork(...)`
   under shared lock and reconstructs an owned `DataPageChain` by feeding
   each raw tuple into `DataPageChain::insert_raw_tuple`.
2. **`DiskannScanOpaque`** — scan-owned metadata, chain, grouped-PQ query
   scratch, reusable `VisitedState`, eager `result_buf`, cursor, rescan
   guard, and reloption-derived `top_k` / `list_size` / `rerank_budget`.
3. **`ambeginscan` / `amrescan` / `amgettuple` / `amendscan`** — the
   previous `"not yet implemented (task 17 phase 3)"` stubs in
   `routine.rs` are replaced with a working ordered scan path.
4. **A pg_test smoke** — create index + `ORDER BY ... LIMIT` round-trip
   through `ec_diskann`.

## Implementation notes

- **`ambeginscan`** allocates the opaque in the scan memory context via
  `PgBox::alloc_in_context(PgMemoryContexts::CurrentMemoryContext)`,
  materializes metadata + chain once, snapshots scan reloptions, and
  stores the raw pointer in `IndexScanDesc.opaque`.
- **`amrescan`** enforces the Phase 6B contract:
  `nkeys == 0`, `norderbys == 1`, non-null ORDER BY datum, query array
  decoded from `FLOAT4ARRAYOID`, dimension check against metadata.
- Query prep uses Packet 11031's persisted-codebook helpers:
  `build_grouped_pq_lut_from_persisted`, plus the flat codebook / rotated
  query caches named in the design doc.
- Scan execution delegates to `scan::vamana_scan_with`; no greedy-descent
  or rerank logic is reimplemented in the pgrx lane.
- Prefilter score is `-grouped_pq_score_f32(...)` so the scan shell can
  keep minimizing while the index is maximizing inner product.
- Exact rerank fetches the heap row at `primary_heaptid`, detoasts the
  `ecvector` with `ecvector_datum_to_vec`, and computes the operator-facing
  negative inner product against the raw query.
- **Important:** exact rerank uses raw `-ip`, not the build-side clamped
  helper, so result ordering matches SQL `<#>` semantics and
  `xs_recheckorderby = false` remains truthful.
- **`amgettuple`** only drains the eager `result_buf`, writes
  `xs_heaptid`, clears `xs_recheckorderby`, and advances the cursor.
- **`amendscan`** drops the Rust-owned opaque fields with
  `ptr::drop_in_place` and then frees the palloc'd opaque with
  `pg_sys::pfree`.

## Reloptions

Phase 6B-2 also threads the scan runtime reloptions into
`TqDiskannOptions`:

- `list_size` (default `100`)
- `rerank_budget` (default `64`)
- `top_k` (default `10`)

These are read once at `ambeginscan` and stored on the opaque.

## Smoke / verification

The new pg_test builds a tiny `ecvector` table, creates an `ec_diskann`
index, forces planner preference away from seqscan / bitmapscan / sort,
and verifies:

1. `EXPLAIN` routes through an index scan.
2. `SELECT ... ORDER BY embedding <#> ... LIMIT 2` executes.
3. The first row is the nearest vector under `<#>`.

Verification run for this slice:

```text
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
```

Observed results:

- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — clean apart from the 8 pre-existing
  `unnecessary_sort_by` warnings in `reader.rs`, `scan.rs`, and
  `vamana.rs`
- `cargo test --lib ec_diskann` — `111 passed`, `0 failed`

## Reviewer notes

- **Dense-block assumption:** `DataPageChain::get_page(block_number)`
  indexes pages as a dense sequence starting at
  `FIRST_DATA_BLOCK_NUMBER`. `materialize_chain_from_index` therefore
  assumes the on-disk data blocks are dense from block 1 upward.
  Phase 5C-3 ambuild currently writes exactly that layout, so the
  round-trip is valid in this slice.
- **No `ANALYZE` in the smoke test:** `ANALYZE` reaches the still-stubbed
  `amvacuumcleanup` path (Phase 8B). The smoke intentionally stays on the
  Phase 6B scan surface: create index, route planner, execute ordered scan.
- **Heap fetch pattern:** the rerank closure follows the same
  `table_tuple_fetch_row_version` / slot datum extraction / `ExecClearTuple`
  lifecycle as `ec_hnsw`'s scan-time heap rerank
  (`src/am/ec_hnsw/source.rs:289-323`, `src/am/ec_hnsw/source.rs:447-474`,
  `src/am/ec_hnsw/scan.rs:1494-1515`, `src/am/ec_hnsw/scan.rs:2339-2340`).
- **Page walk reference:** the metadata/data-block materializer mirrors
  `ec_hnsw`'s scan-time block-count + shared-lock page decode pattern
  (`src/am/ec_hnsw/scan.rs:967-970`, `src/am/ec_hnsw/graph.rs:1557-1607`),
  adapted to append raw tuples into an owned `DataPageChain`.

## Not doing in this packet

- **Insert path** — Phase 7.
- **Vacuum callbacks** — Phase 8B.
- **Planner cost model activation** — Phase 9 still leaves the
  disable-cost shim in place.
- **Memory-conflict-surface refresh** — still a later cleanup pass.
