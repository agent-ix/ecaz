# Review Request: Bootstrap Frontier Refill After Consume

Scope:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added scan-local bootstrap refill behavior: after consuming the current frontier head, the scan can refill from the remaining entry-point neighbors while frontier capacity remains.
- Introduced explicit scan state for the bootstrap entry TID so refill can reuse the persisted entry adjacency without adding full traversal state yet.
- Refill uses the existing visited-element set to avoid reseeding already seen candidates.
- Added regression coverage that consuming one head on a wider fixture removes the consumed candidate and backfills exactly one previously unseen entry-neighbor candidate.

Review focus:
- Whether using the visited set plus stored bootstrap entry TID is the right minimal refill mechanism before real graph expansion
- Whether consume-and-refill keeps the current bootstrap frontier coherent without leaking traversal semantics into tuple production
- Whether the new refill regression is strong enough for this stage

Questions to answer:
- Is consume-and-refill from entry adjacency the right next bridge toward real traversal, or does it overfit the entry-point bootstrap?
- Are there any invariants around visited-state or frontier uniqueness that should be strengthened before refill expands beyond this narrow entry-adjacency path?
- Is the explicit bootstrap entry TID the right temporary state seam, or should this already be generalized toward candidate-local adjacency expansion?
