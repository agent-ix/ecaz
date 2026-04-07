# Request: Scheduler-Node-First Consume

Commit: `bc70e6c`

Summary:
- Updates `src/am/scan.rs` so the scheduler-first consume path removes the visible frontier candidate by the beam scheduler’s chosen element TID, rather than first converting that choice back into a transient Vec index.
- Keeps the existing Vec scan fallback for cases where the scheduler has no live visible node.

Files:
- `src/am/scan.rs`

Why this matters:
- The last slice removed cached frontier-head state, but the runtime consume path still converted scheduler authority back into Vec-index authority before removal.
- This slice keeps the scheduler in the lead a bit longer by consuming the visible frontier node identified by the beam directly.
- It is still an intermediate step: the Vec remains the materialization container, but the hot runtime path now depends less on Vec indexing and more on scheduler-owned node identity.

Review focus:
- Whether node-first consume is the right next reduction in Vec-index coupling
- Whether any remaining runtime paths still promote scheduler-chosen nodes back into Vec-index-first logic unnecessarily
- Whether the next ownership-transfer slice should target the remaining `find_candidate_frontier_index` dependency itself
