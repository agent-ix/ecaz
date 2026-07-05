# Request: Clear Order-By Score On Exhaustion And Rescan

Commit: `7bfea12`

Summary:
- `amgettuple` now clears the visible AM order-by null flag before returning `false`, including the empty-index fast path and the normal exhaustion path.
- Adds a debug helper that inspects the order-by score lifecycle across `amrescan`, first tuple production, exhaustion, and a second `amrescan`.
- Adds pg regression coverage that the published order-by score starts empty, becomes non-null after the first tuple, and clears again on exhaustion and on `amrescan`.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- The previous slice made tuple production publish `current_result.score`, but it still left the prior score visible after the scan exhausted.
- That stale descriptor state would make later ordered execution bookkeeping less trustworthy, especially when the same scan descriptor is rescanned and reused.
- This slice makes the order-by output behave like real scan-local result state rather than a one-way latch.

Review focus:
- Whether every `amgettuple` false-return path that should clear order-by output now does so
- Whether the scan descriptor reuse behavior across `amrescan` is now coherent with the emitted order-by contract
- Whether the new lifecycle test covers the meaningful stale-output cases without overfitting to current bootstrap execution details
