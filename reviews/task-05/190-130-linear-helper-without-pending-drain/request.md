# Request: Keep Pending-Result Drain Out Of The Linear Helper

Commit: `95a9d78`

Summary:
- `next_linear_scan_heap_tid` no longer tries to drain previously pending heap TIDs before scanning for the next linear element.
- Pending duplicate emission is now fully owned by the explicit top-level drain step in `amgettuple`, which was introduced in the prior slice.
- This leaves the linear helper responsible only for selecting/materializing the next linear-scan result.

Files:
- `src/am/scan.rs`

Why this matters:
- After `124ff6e`, the staged executor already had an explicit pending-result drain step before bootstrap and linear selection.
- Keeping the old fallback drain branch inside `next_linear_scan_heap_tid` left a dead coupling in the code path and blurred the intended runtime contract.
- Removing it makes the current staged flow easier to reason about: drain current result first, then try bootstrap, then let the linear helper find the next result.

Review focus:
- Whether the staged executor now has a clear enough separation between duplicate drain and next-result selection
- Whether any current path still expects the linear helper to revive or continue an existing pending result
- Whether this cleanup leaves a better base for later work on ordered beam-driven result production
