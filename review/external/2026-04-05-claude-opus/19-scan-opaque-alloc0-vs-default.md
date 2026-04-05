# Review: TqScanOpaque alloc0 vs Default Mismatch

**File:** `src/am/mod.rs:340-355` and `1288-1316`
**Severity:** Low (currently correct by coincidence)
**Category:** Correctness / maintainability

## Finding

`ambeginscan` allocates the scan opaque state with `PgBox::alloc0()`:

```rust
(*scan).opaque = PgBox::<TqScanOpaque>::alloc0().into_pg().cast();
```

`alloc0` zero-fills the allocation. The `TqScanOpaque` struct has a `Default` impl that sets specific initial values:

```rust
impl Default for TqScanOpaque {
    fn default() -> Self {
        Self {
            rescan_called: false,
            query_dimensions: 0,
            query_values: ptr::null_mut(),
            next_block_number: page::FIRST_DATA_BLOCK_NUMBER,  // = 1, not 0!
            next_offset_number: 1,
            scan_exhausted: false,
            pending_heaptids: [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY],
            pending_heaptid_count: 0,
            pending_heaptid_index: 0,
        }
    }
}
```

Key differences between zero-fill and Default:
- `next_block_number`: zero-fill = 0, Default = 1 (FIRST_DATA_BLOCK_NUMBER)
- `pending_heaptids`: zero-fill = all zeros, Default = all INVALID (block=u32::MAX, offset=u16::MAX)

**Currently safe because:** `amrescan` calls `reset_scan_position()` which sets `next_block_number = FIRST_DATA_BLOCK_NUMBER` and `next_offset_number = 1`. And `amgettuple` requires `rescan_called = true`. So the zero-initialized values are never used.

But if anyone adds a code path that reads from `TqScanOpaque` before `amrescan`, the zero values would be wrong. The zero-filled `next_block_number = 0` would point to the metadata page, causing incorrect scan behavior.

## Recommendation

Either:
1. Use `PgBox::alloc0()` followed by writing the Default values (defensive but redundant)
2. Or use palloc + ptr::write to initialize with Default

Option 2:
```rust
let opaque = unsafe { pg_sys::palloc(std::mem::size_of::<TqScanOpaque>()) }.cast::<TqScanOpaque>();
unsafe { ptr::write(opaque, TqScanOpaque::default()) };
(*scan).opaque = opaque.cast();
```

## Action Required

Low priority. Replace `alloc0` with explicit Default initialization to prevent latent bugs from zero-vs-default divergence.
