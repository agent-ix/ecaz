# Task: DataPageChain O(1) Page Lookup

Review: `review/10049-datapage-chain-linear-lookup/request.md`
Priority: batch 1
Status: ready

## Prompt

Replace the linear page lookup in DataPageChain with O(1) index arithmetic.

File: `src/am/page.rs`

Current code at lines 484-495:

```rust
pub fn get_page(&self, block_number: u32) -> Option<&DataPage> {
    self.pages
        .iter()
        .find(|page| page.block_number == block_number)
}

pub fn get_page_mut(&mut self, block_number: u32) -> Option<&mut DataPage> {
    self.pages
        .iter_mut()
        .find(|page| page.block_number == block_number)
}
```

Pages are always contiguous starting from `FIRST_DATA_BLOCK_NUMBER` (defined as 1
at page.rs:14). New pages are appended sequentially (see page.rs:514-520). So
`block_number - FIRST_DATA_BLOCK_NUMBER` gives the Vec index directly.

Replace both methods with:

```rust
pub fn get_page(&self, block_number: u32) -> Option<&DataPage> {
    let index = block_number.checked_sub(FIRST_DATA_BLOCK_NUMBER)? as usize;
    self.pages.get(index)
}

pub fn get_page_mut(&mut self, block_number: u32) -> Option<&mut DataPage> {
    let index = block_number.checked_sub(FIRST_DATA_BLOCK_NUMBER)? as usize;
    self.pages.get_mut(index)
}
```

Add a debug assertion in the page allocation path (the push in add_page or
equivalent) that validates `pages[i].block_number == FIRST_DATA_BLOCK_NUMBER + i`
to catch any future non-contiguous allocation.

These methods are called by `read_element` and `update_element` during the backfill
loop in `flush_build_state` (build.rs:546-570), which runs for every element. For
N elements across P pages, this changes backfill from O(N*P) to O(N).

## Validate

```bash
cargo test
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Branch from current upstream main. Push branch for review.
