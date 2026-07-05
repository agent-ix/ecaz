# Request: Stale Scheduler Node Cleanup

Commit: `0e0b71b`

Summary:
- Updates `src/am/scan.rs` so frontier-head selection and frontier-head consumption purge stale beam-scheduler nodes that no longer map to any visible frontier candidate.
- Adds a focused unit test that seeds one stale queued node and one live visible node, then verifies head recomputation drops the stale node and selects the live one.

Files:
- `src/am/scan.rs`

Why this matters:
- Requests `80` through `82` left the scan path in a dual-structure state: the beam scheduler chooses nodes, but the visible frontier vector still owns materialization data.
- If those two structures drift even briefly, the previous code could keep peeking the same stale scheduler head and repeatedly fail to map it back into the visible frontier.
- This slice makes that intermediate contract defensive instead of optimistic while the frontier still has split ownership.

Review focus:
- Whether purging stale scheduler nodes during head recomputation and consume is the right defensive behavior for the current dual-structure design
- Whether any remaining scheduler-first paths can still get stuck behind an unmappable queued node
- Whether the next slice should keep reducing visible frontier ownership in `src/am/scan.rs` rather than adding more synchronization logic
