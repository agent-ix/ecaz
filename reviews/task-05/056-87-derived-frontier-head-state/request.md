# Request: Derived Frontier Head State

Commit: `b2971cf`

Summary:
- Removes the cached `candidate_frontier_head` field from `TqScanOpaque`.
- Makes `src/am/scan.rs` derive the current frontier head on demand from the scan-owned scheduler first, with the existing Vec scan retained as fallback.
- Updates `src/am/scan_debug.rs` and the scan unit tests to query derived head state directly instead of reading a cached field.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- The current frontier still has split ownership: the Vec carries materialization data while the beam scheduler increasingly owns ordering.
- Keeping a separate cached head index was extra mutable scan bookkeeping on the Vec side, even after scheduler-first selection had become the real contract.
- Removing that cache makes head selection explicit derived state and reduces the amount of frontier authority that still lives outside `src/am/search.rs`.

Review focus:
- Whether deriving head state on demand is the right intermediate contract while the frontier Vec still exists
- Whether any remaining debug/runtime paths still assume head is cached mutable state rather than computed state
- Whether the next ownership-transfer slice should target the remaining Vec-index mapping itself rather than adding new cached frontier metadata
