# Review: amgettuple Reads Metadata Page on Every Call

**File:** `src/am/mod.rs:440-441`
**Severity:** Medium (performance hot path)
**Category:** Optimization

## Finding

Every `amgettuple` call reads the metadata page to check dimensions and compute `code_len`:

```rust
let metadata = read_metadata_page((*scan).indexRelation);
if metadata.dimensions == 0 {
    return false;
}
let code_len = crate::code_len(metadata.dimensions as usize, metadata.bits);
```

`read_metadata_page` acquires a SHARE lock, reads the buffer, decodes 22 bytes, and releases the lock. During a full linear scan of N elements across P pages, this means N+P metadata page reads (one per `amgettuple` call), when the metadata cannot change during the scan (the metadata is immutable after build/first-insert shape is established, and dimensions/bits never change).

## Recommendation

Cache `dimensions`, `bits`, and `code_len` in `TqScanOpaque` during `amrescan` (which already reads metadata at line 395). Then `amgettuple` can use the cached values directly without re-reading the metadata page.

The metadata is already read in `amrescan`:
```rust
let metadata = read_metadata_page((*scan).indexRelation);
```

Simply store the relevant fields in the scan opaque at that point.

## Action Required

Add `dimensions: u16`, `bits: u8`, and `code_len: usize` fields to `TqScanOpaque` and populate them in `amrescan`. Remove the per-call `read_metadata_page` from `amgettuple`.
