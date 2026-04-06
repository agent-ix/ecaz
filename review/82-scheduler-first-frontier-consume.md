# Request: Scheduler-First Frontier Consume

Commit: `5009995`

Summary:
- Makes `src/am/scan.rs` consume the visible candidate frontier through the beam scheduler's current best queued node before falling back to the cached/vector head index.
- Factors visible-candidate lookup by element TID into a shared helper so recompute and consume use the same scheduler-to-frontier mapping.
- Updates the debug/test seam so consume/refill helpers report the actually consumed candidate instead of deriving it from the cached head index.
- Tightens unit and pg regression coverage around scheduler-driven consume behavior.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Why this matters:
- The executor was already beam-first for head selection, but still vector-index-first for actual consumption.
- This slice makes the scheduler more authoritative over real frontier mutation, which is the next required step before moving broader frontier ownership behind `src/am/search.rs`.
- The debug-helper change keeps the review/test surface aligned with actual runtime behavior instead of preserving stale cached-head assumptions.

Review focus:
- Whether scheduler-first consume with cached-index fallback is the right intermediate contract
- Whether the debug helper now exposes the correct “actually consumed” candidate semantics for pg regressions
- Whether any remaining vector-index assumptions should move next into the shared search seam
