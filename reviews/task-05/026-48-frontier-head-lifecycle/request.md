# Review Request: Frontier Head Lifecycle

Scope:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Extended the two-slot frontier lifecycle so full bootstrap scan exhaustion clears candidate-frontier state as well as current-result state.
- Added a focused regression helper that snapshots the frontier head and both slots after `amrescan`, after partial tuple production, and after full exhaustion.
- Added coverage that the current frontier head remains stable during partial bootstrap linear-scan progress and only clears when the scan is fully exhausted.

Review focus:
- Whether clearing candidate-frontier state on full bootstrap scan exhaustion is the right invariant for later traversal work
- Whether the lifecycle helper and regression capture the intended current semantics without overcommitting future traversal behavior
- Whether any additional lifecycle edge around partial progress or exhaustion should be covered before frontier advancement starts mutating head state

Questions to answer:
- Is it correct for the current frontier head to remain unchanged through partial linear-scan progress?
- Is full-exhaustion frontier clearing the right boundary, or should frontier state survive exhaustion until a later explicit reset?
- Are there missing invariants around how current-result state and frontier state should clear together?
