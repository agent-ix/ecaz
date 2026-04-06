# 05 — `amendscan` casts opaque seven times

**Severity:** Low (style)  
**File:** `src/am/scan.rs:180-186`

## Finding

Each `free_*` call in `amendscan` independently casts `opaque` to `&mut TqScanOpaque`:

```rust
free_scan_candidate_frontier(&mut *opaque.cast::<TqScanOpaque>());
free_bootstrap_expansion(&mut *opaque.cast::<TqScanOpaque>());
free_scan_expanded_set(&mut *opaque.cast::<TqScanOpaque>());
free_scan_visited_set(&mut *opaque.cast::<TqScanOpaque>());
free_scan_emitted_set(&mut *opaque.cast::<TqScanOpaque>());
free_scan_prepared_query(&mut *opaque.cast::<TqScanOpaque>());
free_scan_query(&mut *opaque.cast::<TqScanOpaque>());
```

## Concrete concern

Each cast creates a separate `&mut` reference to the same memory. This is fine because the borrows don't overlap in time, but a single cast-once pattern is more obviously correct:

```rust
let opaque = &mut *opaque.cast::<TqScanOpaque>();
free_scan_candidate_frontier(opaque);
free_bootstrap_expansion(opaque);
// ...
```

The repeated cast pattern reduces confidence during unsafe audit and is a potential maintenance hazard if someone adds a call between two `free_*` calls that holds a reference.

## Impact

Style only. No correctness or performance issue.
