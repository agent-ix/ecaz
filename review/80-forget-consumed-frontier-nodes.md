# Request: Forget Consumed Frontier Nodes

Commit: `2853b29`

Summary:
- Adds `BeamSearch::forget_queued(node)` in `src/am/search.rs`.
- Uses it from `src/am/scan.rs` when a bootstrap frontier head is consumed, so scan-owned beam state stops carrying that node after visible frontier consumption.
- Adds focused unit coverage on both the pure search helper and the scan-side alignment behavior.

Files:
- `src/am/scan.rs`
- `src/am/search.rs`

Why this matters:
- It tightens the contract between the visible candidate frontier and the scan-owned beam scheduler.
- It removes one of the remaining cases where scan execution had to tolerate stale scheduler entries instead of keeping both structures aligned.
- It is a small but real move of frontier ownership toward the shared search seam.

Review focus:
- Whether `forget_queued` has the right semantics for queued-vs-expanded nodes
- Whether removing consumed nodes from the beam scheduler creates any rescan/refill edge cases under the current bootstrap execution model
- Whether the new alignment test is strong enough for this boundary
