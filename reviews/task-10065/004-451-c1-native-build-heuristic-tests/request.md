# Review Request: C1 Native Build Heuristic Tests

Current head at execution: `8a1ca68`

## Context

This checkpoint continues the native-builder stabilization work after packet
`450`. The goal here is simple: pin down the remaining native helper behavior
with direct tests so the branch is easier to land without depending on recall
packets alone.

This slice does not change runtime heuristics. It only adds focused regression
coverage around the native BUILD helper behavior already in the tree.

## What changed

Added three native-builder tests in `src/am/build.rs`:

1. `flatten_native_neighbor_slots_dedups_and_skips_origin`
   - proves flattening keeps first-seen layer order
   - skips self-links
   - removes duplicates across layers

2. `add_native_backlinks_uses_free_slot_before_rewrite`
   - proves backlink insertion prefers a free slot when the target layer still
     has capacity
   - avoids unnecessary slice rewrite in the common cheap case

3. `add_native_backlinks_rewrites_full_slice_for_better_candidate`
   - proves a full backlink slice is rescored and rewritten when the new node
     outranks an existing neighbor
   - locks in the current replacement ordering through the shared
     `select_best_backlink_candidates` helper

These tests cover the exact helper paths the reviewer previously called out as
important to pin down for native BUILD.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Note: as with packet `450`, an initial parallel validation run caused the plain
`cargo test` lane to be invalidated by the pg17 wrapper’s extension rebuild.
The final `cargo test` result cited here is from the clean standalone rerun.

## Review focus

1. Are these the right heuristic tests to treat the current native helper
   behavior as pinned for merge?
2. Do you want one more direct helper test in this branch for the upper-layer
   walk result shape, or is the current coverage enough to move focus back to
   recall evidence / final merge readiness?
