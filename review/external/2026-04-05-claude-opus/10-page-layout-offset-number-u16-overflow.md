# Review: offset_number u16 Overflow in DataPage

**File:** `src/am/page.rs:361-377`
**Severity:** Low (theoretical, unlikely with 8KB pages)
**Category:** Correctness / edge case

## Finding

`insert_raw_tuple` casts `tuples.len()` to `u16` for the offset number:

```rust
Ok(ItemPointer {
    block_number: self.block_number,
    offset_number: self.tuples.len() as u16,
})
```

If more than 65,535 tuples were inserted into a single `DataPage`, this would silently overflow. In practice, with 8KB pages and minimum tuple sizes (alignment + header = 8 bytes), the maximum is ~1000 tuples per page, so this cannot happen with the current page size.

However, the `DataPageChain` is used during build where the in-memory representation doesn't enforce physical page size limits the same way PostgreSQL does. The `can_fit_raw_tuple` check prevents overflow indirectly, but there's no explicit assertion.

## Recommendation

Add a debug_assert or checked conversion:

```rust
offset_number: u16::try_from(self.tuples.len())
    .expect("tuple count should fit in u16"),
```

## Action Required

Low priority. Replace the `as u16` cast with `try_from` to fail explicitly rather than silently wrapping.
