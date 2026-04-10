# Task: Build Element/Neighbor Locality

Review: `review/10044-build-element-neighbor-locality/request.md`
Priority: batch 1
Status: ready

## Prompt

Improve page locality in `flush_build_state` by interleaving element and neighbor
tuple writes instead of writing all elements first and all neighbors second.

File: `src/am/build.rs`, function `flush_build_state` (line 523)

Current structure (lines 532-570):

```rust
// Loop 1: insert ALL element tuples across pages
for (idx, tuple) in state.heap_tuples.iter().enumerate() {
    let element_tid = data_pages.insert_element(...);
    element_tids.push(element_tid);
}

// Loop 2: insert ALL neighbor tuples, then backfill element.neighbortid
for (idx, element_tid) in element_tids.iter().copied().enumerate() {
    let neighbor_tid = data_pages.insert_neighbor(...);
    let mut element = data_pages.read_element(element_tid, ...);
    element.neighbortid = neighbor_tid;
    data_pages.update_element(element_tid, &element);
}
```

This puts all elements on early pages and all neighbors on later pages. When
graph traversal loads an element then immediately loads its neighbors, they are
on different pages — poor cache locality.

Rewrite to interleave: for each node, write the neighbor tuple first (to get its
TID), then write the element tuple with `neighbortid` already set. This eliminates
the backfill loop entirely.

**Important constraint:** `neighbor_refs` reference `element_tids` of OTHER nodes.
In the current two-loop design, all element TIDs are known before any neighbor
is written. In the interleaved design, when writing node N's neighbors, nodes
with index > N don't have element TIDs yet.

Two approaches to handle this:

**(a) Two-pass with pre-allocated TIDs:** first pass reserves space for all
element+neighbor pairs and records their TIDs without writing content.
Second pass fills in the actual data. This is more complex.

**(b) Keep two loops but change allocation strategy** so each node's element and
neighbor land on the same page or adjacent pages. For example, reserve space for
both when inserting the element. If both fit on the current page, they'll be
co-located.

Choose whichever approach you find cleanest. The goal is: after build, an
element tuple and its neighbor tuple should be on the same page (or adjacent
pages) rather than separated by hundreds of pages.

The backfill loop (lines 563-569) also uses `read_element` and `update_element`
which go through `DataPageChain::get_page` — if task 10049 hasn't landed yet,
those are O(N*P) linear lookups. If you're doing this after 10049, they're O(1).
Either way, eliminating the backfill loop entirely is ideal.

## Validate

```bash
cargo test
cargo pgrx test pg17
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Branch from current upstream main. Push branch for review.
