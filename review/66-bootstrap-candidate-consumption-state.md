# Request: Bootstrap Candidate Consumption State

Commit: `251e579`

Summary:
- Makes `amgettuple` consume one bootstrap frontier candidate into explicit scan-owned active-candidate state before the existing linear tuple scan runs.
- Clears that active candidate on rescan and exhaustion.
- Updates lifecycle expectations so partial scan progress is allowed to advance bootstrap frontier state.

Files:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- Seeded frontier candidates are no longer purely decorative bootstrap state.
- The scan state machine now has a real bridge between candidate selection and later graph-driven execution without changing planner-visible tuple ordering yet.

Review focus:
- Whether `maybe_consume_bootstrap_frontier_candidate` is the right execution boundary for this stage
- Interaction between active-candidate state, frontier refill, and duplicate heap-tid draining
- Reset/cleanup correctness on exhaustion and `amrescan`
- Whether the updated lifecycle tests now reflect the intended bootstrap semantics
