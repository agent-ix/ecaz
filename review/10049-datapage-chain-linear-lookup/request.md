# Review Request: DataPageChain Linear Lookup During Build

Scope:
- `src/am/page.rs` — `DataPageChain::get_page`, `DataPageChain::get_page_mut`

## Problem

`get_page` and `get_page_mut` (page.rs:484-495) do a linear scan through all pages to find a
page by block number:

```rust
pub fn get_page(&self, block_number: u32) -> Option<&DataPage> {
    self.pages.iter().find(|page| page.block_number == block_number)
}
```

During build, the backfill loop (build.rs:563-569) calls `read_element` and `update_element` for
every element, which both call `get_page` / `get_page_mut`. For N elements spread across P pages,
the backfill is O(N * P).

Pages are always contiguous starting from `FIRST_DATA_BLOCK_NUMBER` (page.rs:476, 514-520), so
a direct index lookup is possible:

```rust
pub fn get_page(&self, block_number: u32) -> Option<&DataPage> {
    let index = block_number.checked_sub(FIRST_DATA_BLOCK_NUMBER)? as usize;
    self.pages.get(index)
}
```

## Impact

Affects **build time** (and therefore test init). For a 10K-element index with ~100 pages, the
current approach does ~1M comparisons during backfill. With O(1) lookup it's ~10K.

For small test fixtures this is negligible, but it scales poorly for larger builds.

## Suggested Fix

Replace the linear search with index arithmetic as shown above. Add a debug assertion that
`pages[i].block_number == FIRST_DATA_BLOCK_NUMBER + i` to catch any future non-contiguous
allocation.
