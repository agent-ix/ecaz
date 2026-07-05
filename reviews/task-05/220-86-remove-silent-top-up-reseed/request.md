# Request: Remove Silent Top-Up Reseed

Commit: `90a2761`

Summary:
- Updates `src/am/scan.rs` so `top_up_bootstrap_frontier` no longer silently scrapes the visible frontier back into the beam scheduler when the scheduler is empty.
- Makes the affected helper tests seed the scheduler explicitly when they want to exercise real top-up behavior.
- Adds a focused unit test that locks in the new contract: top-up does nothing when the scheduler is empty instead of hiding the inconsistency.

Files:
- `src/am/scan.rs`

Why this matters:
- Request `84` called out the silent Vec-to-beam recovery inside top-up as a safety net that can hide real beam-state bugs.
- Discovery seeding, persistent scan-owned scheduler state, and stale-node cleanup are all in place now, so this fallback was starting to work against the ownership transfer toward `src/am/search.rs`.
- Removing it makes the beam scheduler contract explicit and easier to reason about during the remaining dual-structure phase.

Review focus:
- Whether removing the silent top-up reseed is the right line now that direct seeding and persistent scheduler state are both in place
- Whether any remaining helper/runtime paths still depend on implicit Vec-to-beam recovery
- Whether the next slice should keep moving frontier ownership behind the search seam instead of adding more synchronization fallbacks
