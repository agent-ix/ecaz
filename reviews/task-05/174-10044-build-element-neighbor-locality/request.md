# Review Request: Build Element-Neighbor Page Locality

Scope:
- `src/am/build.rs` — `flush_build_state`
- `src/am/page.rs` — `DataPageChain`

## Problem

FR-007 states:
> "Element and neighbor tuples for the same node SHOULD be placed on the same page when space permits (locality for scan)"

The current build serialization violates this. `flush_build_state` (build.rs:532-569) runs two
sequential loops:

1. First loop: inserts ALL element tuples into the DataPageChain
2. Second loop: inserts ALL neighbor tuples into the DataPageChain

This packs elements onto early pages and neighbors onto later pages. A node's element on page 2
will have its neighbor tuple on a much later page.

## Impact

During graph traversal, every node visit requires two buffer reads:

1. `load_graph_element` reads the element tuple (page X)
2. `load_graph_neighbors` reads the neighbor tuple (page Y, where Y >> X)

For ef_search=64 with average expansion, this is ~128 random page reads per query instead of ~64
with co-located tuples. The I/O amplification doubles for indexes that exceed the buffer cache.

This also affects **build time in tests** — `write_data_pages` writes each staged page as a separate
GenericXLog transaction, and spreading the same logical node across two distant pages means the OS
page cache gets no locality benefit during the verification reads that follow build.

## Suggested Fix

Interleave element and neighbor insertion: for each node, insert the element tuple then immediately
insert its neighbor tuple so they land on the same page when space permits. The backfill of the
element's `neighbortid` field can use an in-page update since both tuples are on the same
`DataPage`.

Sketch:
```
for (idx, tuple) in heap_tuples.iter().enumerate() {
    let neighbor_tuple = build_neighbor_tuple(idx, &graph_nodes, &element_tids);
    let neighbor_tid = data_pages.insert_neighbor(&neighbor_tuple)?;
    let element = TqElementTuple { neighbortid: neighbor_tid, ... };
    let element_tid = data_pages.insert_element(&element)?;
    element_tids.push(element_tid);
}
```

This requires knowing the neighbor TIDs before inserting elements. Since all element TIDs are needed
to resolve neighbor pointers, this means a two-phase approach:

1. First pass: assign logical IDs, build the neighbor-to-element mapping
2. Second pass: interleave element + neighbor insertion with resolved TIDs

Alternatively, insert element + empty neighbor on the same page, collect all element TIDs, then do
an in-place update pass on the neighbor tuples (which are already co-located).

Please review whether co-location is worth the added build complexity at this stage, or whether it
should be deferred until scan I/O becomes the bottleneck.
