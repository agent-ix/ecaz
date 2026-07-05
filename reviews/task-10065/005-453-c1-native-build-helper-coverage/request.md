# Review Request: C1 Native Build Helper Coverage

Current head at execution: `920ef8e`

## Context

This checkpoint adds direct regression coverage for the native BUILD helper
behavior that was previously only exercised indirectly through larger build and
scan tests.

The goal is to make the current native serial builder easier to land by pinning
the most fragile helper logic explicitly:

- neighbor flattening
- backlink free-slot admission
- backlink full-slice replacement

No runtime heuristics changed in this slice.

## What changed

Added three focused tests in `src/am/build.rs`:

1. `flatten_native_neighbor_slots_dedups_and_skips_origin`
   - confirms flattening skips self-links
   - confirms duplicates across layers collapse to first-seen order

2. `add_native_backlinks_uses_free_slot_before_rewrite`
   - confirms a target layer with spare capacity inserts the new backlink
     without rewriting the existing slice

3. `add_native_backlinks_rewrites_full_slice_for_better_candidate`
   - confirms a full backlink slice is rescored through the shared
     `select_best_backlink_candidates` ordering
   - confirms the new node is admitted when it outranks an existing backlink

These tests intentionally stay small and deterministic so they lock the current
native helper semantics without requiring another recall lane.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

As in the previous two checkpoints, the final cited `cargo test` result comes
from a clean standalone rerun after the pg17 wrapper invalidated an earlier
parallel plain-test lane by rebuilding/installing the extension.

## Review focus

1. Is this enough direct helper coverage for the current native BUILD semantics,
   or do you still want a dedicated upper-layer seed/frontier shape test before
   merge?
2. With packets `450` and `453` in place, is the remaining branch risk mostly
   recall evidence rather than helper-level ambiguity?
