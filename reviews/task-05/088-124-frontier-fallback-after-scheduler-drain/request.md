# Request: Preserve Visible-Frontier Fallback After Scheduler Drain

Commit: `7bc1788`

Summary:
- Adds an explicit comment on `current_candidate_frontier_head` documenting that the scheduler tracks unexpanded expansion sources, not every still-visible result candidate.
- Adds a regression test proving that the visible frontier can remain non-empty after the scheduler has drained, and that head derivation must still fall back to the visible frontier in that state.

Files:
- `src/am/scan.rs`

Why this matters:
- A tempting cleanup is to make the scheduler fully authoritative for head selection and consume, but current bootstrap fill does not support that contract yet.
- After the scheduler consumes candidates as expansion sources, the visible frontier may still retain those candidates for later emission.
- This checkpoint records that boundary explicitly so later graph-search refactors do not accidentally collapse the two structures too early.

Review focus:
- Whether the new regression test captures the real current invariant without overfitting to test-only setup
- Whether the comment explains the boundary clearly enough for later scan/search ownership work
- Whether there are any runtime paths where the scheduler should still be authoritative even after visible-frontier candidates remain
