# Review Request: Wider Bootstrap Frontier Seeding

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- The bootstrap frontier now seeds the entry candidate plus up to two live neighbors from the entry point's persisted adjacency.
- The frontier still does not perform graph traversal or dynamic expansion during `amgettuple`; this slice only widens the initial candidate set available after `amrescan`.
- Debug/test support now exposes the full seeded frontier slot list while preserving the existing narrow two-slot snapshot for older lifecycle checks.
- The visited-state regression now derives expected tids from the full seeded frontier rather than only the first two snapshot slots.

Review focus:
- Whether seeding up to two live neighbors is the right next step before real traversal expansion logic
- Whether the widened frontier remains coherent with current head selection, visited seeding, and bootstrap linear scan behavior
- Whether the split between the legacy two-slot snapshot and the new full-slot debug output is reasonable for this transitional stage

Questions to answer:
- Is the cap of three total seeded bootstrap candidates a sensible narrow bridge toward real traversal, or should this stage stay at two until dynamic expansion exists?
- Are there any ordering or lifecycle edges where widened seeding now creates ambiguity in the current bootstrap scan semantics?
- Is the temporary mismatch between the full internal frontier and the older two-slot snapshot acceptable, given that the new tests read the full slot list where widened behavior matters?
