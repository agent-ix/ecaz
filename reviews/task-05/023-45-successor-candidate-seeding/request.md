# Review Request: Successor Candidate Seeding

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Added one explicit successor-candidate slot in scan-owned state alongside the seeded entry candidate.
- `amrescan` now seeds that successor candidate from the first live neighbor reachable through the entry point's persisted flat adjacency list, when such a neighbor exists.
- This still does not introduce a general candidate heap, visited set, or graph traversal loop; it is only one adjacency-derived follow-on slot.
- Added regression coverage that the seeded successor candidate, when present, points at one of the persisted entry-point neighbor refs and carries a computed score.

Review focus:
- Whether one adjacency-derived successor slot is the right next bridge from seeded entry point state toward a later frontier
- Whether choosing the first live persisted neighbor is a reasonable placeholder without overcommitting to traversal order
- Whether the test handles the current flat-neighbor layout honestly without inventing guarantees about graph density

Questions to answer:
- Is this one-slot successor state a sane intermediate step before introducing a real candidate frontier?
- Is selecting the first live persisted neighbor acceptable for now, given that traversal ordering is not implemented yet?
- Are there missing cases around deleted neighbors, empty adjacency, or rescan/exhaustion lifecycle that should be covered before expanding this into a frontier?
