# Request: Queued Beam Candidate Lookup

Commit: `d44b0bc`

Summary:
- Adds `BeamSearch::queued_candidate(node)` in `src/am/search.rs`.
- Exposes a cheap, non-mutating way to ask whether a node is still represented in the queued beam frontier and, if so, retrieve its exact `BeamCandidate`.
- Adds focused unit coverage that queued lookup does not mutate scheduler order and that expanded or forgotten nodes are not reported as still queued.

Files:
- `src/am/search.rs`

Why this matters:
- The recent scan/search refactor keeps moving ownership away from scan-side helper loops and toward the shared beam structure.
- That transition needs read-only scheduler introspection, not just mutating operations like `peek_best_matching`, `take_best_matching`, and `forget_queued`.
- This helper gives later scan-side cleanup or identity-transfer slices a direct beam-owned lookup API without snapshotting or mutating the scheduler as a side effect.

Review focus:
- Whether `queued_candidate` is the right minimal non-mutating beam lookup seam
- Whether the added API preserves clear ownership boundaries instead of encouraging scan-side peeking into too much scheduler internals
- Whether this helper is the right precursor for future visible-frontier/beam unification work
