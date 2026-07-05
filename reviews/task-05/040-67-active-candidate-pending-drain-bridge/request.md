# Request: Active Candidate Pending-Drain Bridge

Commit: `1cf5968`

Summary:
- Adds the first direct bridge from active bootstrap candidate state into the existing current-result and pending-heap-tid drain machinery.
- Keeps planner-visible tuple production unchanged for now: the new bridge is exercised through scan helpers and regression coverage, not through a changed `amgettuple` result order.

Files:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- This is the missing seam between bootstrap candidate selection and the already-tested duplicate-drain result path.
- It lowers risk for a later switch from pure linear tuple production to candidate-driven tuple production because the drain machinery is now reusable from both sides.

Review focus:
- Correctness of `materialize_active_candidate_result`
- Whether active-candidate clearing and pending-heaptid population happen at the right boundary
- Interaction with duplicate-coalesced elements
- Whether keeping this helper-only for now is the right staging step before changing visible scan order
