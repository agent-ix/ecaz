# Request: Beam-Native Frontier Head Selection

Commit: `86e45ec`

Summary:
- Changes `src/am/scan.rs` so frontier-head selection stays in `BeamCandidate<ItemPointer>` form longer instead of immediately collapsing to a node TID.
- Adds a `current_candidate_frontier_head` helper that returns the shared beam candidate from either the scheduler-first path or the visible-frontier score fallback.
- Keeps the TID-only surface as a thin boundary helper for debug and test code.

Files:
- `src/am/scan.rs`

Why this matters:
- The visible frontier is already stored as `BeamCandidate<ItemPointer>`.
- Scheduler-first consume, stale-node cleanup, active bootstrap state, and removal already operate in beam-native form.
- Head selection was still projecting down to a TID early and then rebuilding around that choice in the consume path. This slice removes that extra projection in the hot path.

Review focus:
- Whether frontier-head selection is now consistently beam-native across scheduler-first and fallback score-order paths
- Whether any remaining runtime consumers still force an early collapse from shared beam candidate state down to raw node identity
- Whether the next ownership transfer should target result materialization state in `src/am/scan.rs` or move more visible-frontier identity behind `src/am/search.rs`
