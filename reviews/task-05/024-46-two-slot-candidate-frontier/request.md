# Review Request: Two-Slot Candidate Frontier

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Replaced the separate entry-candidate and successor-candidate fields in scan state with one explicit fixed two-slot frontier container.
- Kept the behavior the same: slot 0 is the seeded entry candidate and slot 1 is the adjacency-derived successor candidate when present.
- Added regression coverage that `amrescan` builds a two-slot frontier shape where the first slot is always the seeded entry candidate and the second slot is either a concrete scored successor or a cleared empty slot.
- This is still not a general candidate heap or traversal queue; it is only a structured home for the two current candidate slots.

Review focus:
- Whether a fixed two-slot frontier is the right intermediate state before introducing a real candidate collection
- Whether preserving the current entry/successor behavior inside the new container keeps the refactor honest and low-risk
- Whether the new coverage captures the frontier shape clearly enough before the next expansion

Questions to answer:
- Is this two-slot frontier the right staging structure before a larger frontier or priority queue lands?
- Are there any lifecycle edges that should be checked now that entry and successor live in the same container?
- Does the fixed frontier shape still communicate clearly that traversal ordering semantics are not implemented yet?
