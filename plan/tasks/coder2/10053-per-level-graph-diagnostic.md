# Task: Per-Level Graph Structure Diagnostic

Motivation: Review 212 discovered that the build was collapsing the HNSW hierarchy
to level 0. Once the fix lands, coder-1 needs to validate the fixed hierarchy's
structure. There is no diagnostic that shows the full level distribution or
per-level connectivity of the persisted graph.
Priority: batch 2
Status: done

## Prompt

Add a SQL-callable diagnostic surface that reports the persisted HNSW graph's
hierarchical structure. This should be a `pg_test`-gated function (similar to the
existing recall probe surfaces in `src/lib.rs`).

Surface: `tests.tqhnsw_graph_hierarchy_summary(index_regclass regclass)`

Returns a table with columns:
- `level` (integer): the HNSW level
- `node_count` (integer): number of element tuples at this level
- `avg_neighbor_count` (float): average number of valid (non-INVALID) neighbor
  TIDs in the neighbor tuple for this level's neighbor slots
- `min_neighbor_count` (integer): minimum valid neighbors at this level
- `max_neighbor_count` (integer): maximum valid neighbors at this level
- `expected_max_neighbors` (integer): the theoretical max for this level
  (2*m for level 0, m for upper levels)

Implementation:
- Scan all data pages in the index relation
- For each element tuple (tag `0x01`), read its level and neighbor TID
- For each neighbor tuple, use `layer_slot_bounds(level, m, layer)` to extract
  per-layer neighbor lists
- Count valid (non-INVALID) TIDs per layer
- Aggregate by level

Use the existing page scanning pattern from `shared::count_element_tuples` or
`find_duplicate_element_tid` in `insert.rs` — iterate blocks from
`FIRST_DATA_BLOCK_NUMBER`, read each page with SHARE lock, decode tuples.

The metadata page (block 0) provides `m` and `max_level`. Use
`shared::read_metadata_page` to get these.

Expected output on a healthy 10k graph with m=8:

```
level | node_count | avg_neighbor_count | expected_max_neighbors
------+------------+--------------------+-----------------------
    0 |       9600 |               14.2 |                    16
    1 |        350 |                7.1 |                     8
    2 |         40 |                6.5 |                     8
    3 |          8 |                4.2 |                     8
    4 |          2 |                1.5 |                     8
```

Expected output on the BROKEN (pre-212-fix) graph:

```
level | node_count | avg_neighbor_count | expected_max_neighbors
------+------------+--------------------+-----------------------
    0 |      10000 |               12.3 |                    16
```

This immediately shows whether the hierarchy is healthy or collapsed.

File: `src/lib.rs` (add alongside existing recall probe surfaces, gated behind
`#[cfg(any(test, feature = "pg_test"))]`)

Helper functions for page iteration: `src/am/shared.rs`, `src/am/page.rs`

## Validate

```bash
cargo test
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Branch from current upstream main. Push branch for review.

## Completion Notes

**Branch:** `coder2/10053-per-level-graph-diagnostic`

**Implementation:** Added `tqhnsw_graph_hierarchy_summary(index_oid)` as a `#[pg_extern]` function
inside the `#[cfg(any(test, feature = "pg_test"))]` `tests` module in `src/lib.rs`.

**Approach:**
- Uses `open_valid_tqhnsw_index` for AM validation (same pattern as cost/planner snapshot functions)
- Uses `debug_index_pages` to read all data pages (reuses existing test infrastructure)
- Builds a neighbor TID → decoded `TqNeighborTuple` map
- Iterates element tuples, for each layer 0..=element.level computes valid neighbor count
  using the same slot bounds logic as `layer_slot_bounds` in `graph.rs`
- Aggregates per-level: node_count, avg/min/max neighbor count, expected_max_neighbors

**Decisions:**
- Reimplemented `layer_slot_bounds` logic inline (3 lines) rather than making the private
  function in `graph.rs` public, to avoid changing files outside task scope
- Used `debug_index_pages` rather than raw page scanning to stay consistent with existing
  test patterns and avoid duplicating buffer management code
- Stored decoded `TqNeighborTuple` in the map rather than raw bytes to avoid lifetime issues

**Validation:**
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: clean pass
- `cargo test`: all tests pass (2 passed, 20 ignored, 13 size_of_assertions passed)
