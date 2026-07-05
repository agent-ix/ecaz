# Review Request: Scan Visited Seed State

Scope:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added scan-owned visited-element state as a heap-allocated `HashSet<ItemPointer>` referenced from scan opaque.
- Reset/clear the visited set on `amrescan`, seed it from the currently valid frontier candidates after entry/successor seeding, and free it on `amendscan`.
- Added focused regression coverage that the seeded visited set matches the valid frontier candidate tids and remains stable through the current bootstrap linear scan and exhaustion path.

Review focus:
- Whether the visited-set ownership and lifecycle are right for upcoming traversal work
- Whether seeding from currently scored frontier candidates is the right minimal invariant at this stage
- Whether keeping the visited set stable through the bootstrap scan is the right boundary before a real graph walk starts mutating it

Questions to answer:
- Is a heap-allocated `HashSet<ItemPointer>` the right first visited-state representation for scan-owned traversal state?
- Is seeding from valid frontier candidates too eager, or the right precursor to real traversal expansion?
- Are there any missing lifecycle edges around rescan or amendscan that should be covered before candidate-heap work starts?
