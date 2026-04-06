# Request: Beam-Owned Frontier Head Selection

Commit: `1033d90`

Summary:
- Makes `src/am/scan.rs` prefer the scan-owned beam scheduler's current best queued node when recomputing `candidate_frontier_head`.
- Keeps the previous vector scan as a fallback if the scheduler and visible frontier cannot be matched.
- Adds a focused unit test that proves scheduler state can drive visible frontier-head choice.

Files:
- `src/am/scan.rs`

Why this matters:
- This is another ownership shift from ad hoc scan-local arbitration toward the shared search structure.
- It reduces duplicated “best candidate” logic in the executor and makes the beam scheduler more authoritative without yet deleting the vector frontier.
- The fallback keeps this slice safe while the executor is still in a hybrid state.

Review focus:
- Whether the scheduler-first / vector-fallback policy is the right intermediate contract
- Whether the new unit test captures the intended ownership shift clearly enough
- Whether any remaining frontier-head logic should now move fully behind the search seam
