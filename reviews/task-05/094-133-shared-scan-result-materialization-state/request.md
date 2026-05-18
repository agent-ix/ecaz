# Request: Share Result-State Materialization Across Bootstrap And Linear Paths

Commit: `aec8bd9`

Summary:
- Both bootstrap candidate materialization and linear element materialization now feed one shared helper that writes `current_result` plus pending heap-TID drain state.
- The slice also adds a unit regression that pins that helper as the common result-state contract.
- This removes another remaining difference between graph-side and linear-side staged result production.

Files:
- `src/am/scan.rs`

Why this matters:
- The staged executor now already has one explicit pending-drain step and one shared “materialize next result” step.
- Before this slice, bootstrap and linear paths still seeded `current_result` and pending heap TIDs separately.
- Sharing that state transition makes the runtime contract tighter and should make later ordered-result work less likely to diverge across the two paths.

Review focus:
- Whether the new shared helper captures the right current-result contract for both bootstrap and linear materialization
- Whether the helper keeps the current staging model honest without hiding path-specific semantics that still matter
- Whether this leaves the executor in a cleaner state for the next real ordered-traversal slice
