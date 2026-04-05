# Review Request: Skip Invalid Successor Neighbor Refs

Scope:
- `src/am/scan.rs`

What changed:
- Refactored successor-candidate seeding through a small helper that filters `INVALID` neighbor TIDs before attempting any element loads.
- Kept the existing live-neighbor selection semantics otherwise unchanged: continue until one neighbor produces a concrete candidate, then stop.
- Added focused unit coverage that proves `INVALID` refs are skipped and that seeding falls through to the next usable neighbor candidate.

Review focus:
- Whether skipping `INVALID` neighbor refs at the seeding boundary is the right defensive fix for the current flat-adjacency traversal scaffold
- Whether the helper keeps the current “first usable neighbor wins” placeholder semantics clear
- Whether this is the right layer to absorb the invalid-ref guard before fuller traversal state arrives

Questions to answer:
- Is `src/am/scan.rs` the right place for this guard, or should it eventually move into the shared graph-read surface?
- Does the helper capture the current successor-seeding policy cleanly enough for later frontier expansion?
- Are there any other invalid-ref edges in the scan bootstrap path that should be guarded at the same time?
