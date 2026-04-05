# Review Request: `amgettuple` State Gating

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amgettuple` no longer immediately falls through to the generic build-only error.
- It now verifies that the scan descriptor exists, that opaque scan state exists, and that `amrescan` has been called first.
- After those checks, it still rejects actual tuple production with a narrow "not implemented yet" error.

Review focus:
- Scan-callback state machine correctness
- Error-surface coherence between `ambeginscan`, `amrescan`, and `amgettuple`
- Whether the current gating is the right narrow boundary before real result iteration exists

Questions to answer:
- Are there any scan lifecycle paths where the new `amgettuple` checks are still too weak?
- Are the two failure modes distinct and useful enough for debugging executor behavior?
- Are there missing tests around null scan descriptors, missing opaque state, or repeated rescans?
