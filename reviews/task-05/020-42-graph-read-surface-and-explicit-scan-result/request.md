# Review Request: Graph Read Surface And Explicit Scan Result

Scope:
- `src/am/mod.rs`
- `src/am/graph.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Extracted shared page-graph read helpers into `src/am/graph.rs` so scan-side graph access no longer decodes element and neighbor tuples ad hoc.
- Exposed the low-level page line-pointer helpers from `mod.rs` only as far as needed for the new graph-read seam.
- Replaced the scan opaque's scattered current-result fields with one explicit current-result struct carrying element TID, current heap TID, and score.
- Added regression coverage that duplicate draining advances the current-result heap TID while keeping the same element TID and stable score.
- Tightened the neighbor-read tests to assert actual storage invariants instead of assuming every built entry point or first linear-scan result has non-empty adjacency.

Review focus:
- Whether `graph.rs` is the right shared read boundary before real traversal lands
- Whether explicit current-result state is the right minimal result-shaped slot for upcoming candidate/result machinery
- Whether the adjusted tests now capture the real graph-storage guarantees without depending on accidental graph density

Questions to answer:
- Is the new `graph` module boundary strong enough to support both future scan traversal and later graph-aware insert/vacuum reads?
- Is the explicit current-result struct the right minimum shape, or is there a smaller/better representation for the next ordered-scan slice?
- Are there missing graph-read invariants or result-state lifecycle cases that should be covered before candidate heaps or visited-state land?
