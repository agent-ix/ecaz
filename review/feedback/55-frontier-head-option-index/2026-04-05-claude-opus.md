# Feedback: Frontier Head Option Index

Request:
- `review/55-frontier-head-option-index.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is removing the `u8` sentinel now the right precursor to wider frontier growth?

**Yes.** The `u8::MAX` sentinel capped the addressable frontier to 255 slots and conflated "no head" with a magic value. `Option<usize>` is idiomatic Rust, makes the "no valid head" state type-safe (`None`), and removes any artificial width limit. This is the right cleanup before the frontier starts growing beyond the initial seeded pair.

### Remaining slot-oriented assumptions?

No remaining slot-oriented assumptions in the head state. The `candidate_frontier_head: Option<usize>` field (scan.rs:961) is now a plain index into the Vec, with `None` for empty. `recompute_candidate_frontier_head` (scan.rs:529-549) iterates the Vec by index and stores the best index, which is correct for any Vec size.

The debug helpers still use a two-slot `DebugCandidateFrontier` type (scan.rs:1008) for some older snapshot functions, but the newer slot-based debug helpers (`debug_candidate_frontier_slots`, scan.rs:1053) return a `Vec<DebugCandidateSlot>` that works for any frontier width. The older two-slot debug type is legacy from the fixed-slot era and doesn't constrain the real implementation.

### Is keeping the debug snapshot two-slot-shaped acceptable for now?

**Yes, but it's accruing debt.** The two-slot `DebugCandidateFrontier` type alias (scan.rs:1008) is used by some lifecycle debug helpers that predate the Vec-backed frontier. These helpers still work because the frontier is currently seeded with at most two candidates. When wider seeding lands, these helpers will need updating or removal. Not blocking, but worth tracking.

## Additional Findings

No issues found. Clean type-safety improvement.
