# Review Request: Scan Page-Level Neighbor Access

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Added shared scan-side helpers to load an element tuple's persisted neighbor references directly from index pages.
- Added regression coverage that the current scan result exposes concrete neighbor refs and that the metadata entry-point's neighbor refs point at real element tuples in the built index.
- Kept this slice at page-level graph access only; it does not claim true layer-aware traversal because the current on-disk neighbor tuple format stores a flattened adjacency list rather than per-layer segments.

Review focus:
- Whether the new page-read helpers validate tuple slots and tuple bounds defensively enough for upcoming traversal work
- Whether this is the right narrow seam for ordered-scan groundwork given the current flat neighbor tuple layout
- Whether the regression coverage is checking the most important invariants before candidate/result traversal state lands

Questions to answer:
- Are the element and neighbor tuple decode checks sufficient for a shared graph-read primitive at this stage?
- Is it correct to keep the API flat-neighbor-only until the page layout preserves layer boundaries?
- Are there missing edge cases around invalid neighbor tuple counts, invalid offsets, or cross-page adjacency reads that should be covered before traversal starts using these helpers directly?
