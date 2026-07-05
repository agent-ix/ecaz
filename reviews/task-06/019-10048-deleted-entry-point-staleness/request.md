# Review Request: Deleted Entry Point Staleness

Scope:
- `src/am/graph.rs` — `greedy_descend_from_entry`
- `src/am/vacuum.rs`
- `src/am/shared.rs` — metadata page entry_point
- `spec/functional/FR-010-hnsw-vacuum.md`

## Problem

The metadata page stores a single `entry_point` ItemPointer. If the element at that TID is
soft-deleted (via future FR-010 vacuum), graph traversal still starts from it.

`greedy_descend_from_entry` (graph.rs:135-162) loads the entry point element and uses it as the
starting position for greedy descent through upper layers. The deleted-element filter in
`layer0_successor_candidates_from_elements` (graph.rs:536) only filters *neighbors*, not the
entry point itself.

Current state: vacuum is a no-op, so this can't happen today. But once FR-010 is implemented:

1. Entry point element gets all heap TIDs removed by vacuum
2. Vacuum sets `deleted = true` on the element
3. Graph search still starts from this deleted node
4. The entry point's upper-layer neighbors are still traversed (functional but suboptimal)
5. The entry point will never be returned as a result (correct)
6. But if all of the entry point's upper-layer neighbors are also deleted, greedy descent
   degrades to random layer-0 entry (poor recall)

## Impact

Affects **graph traversal quality** after vacuum deletes the entry point or its upper-layer
neighborhood. Recall degrades silently.

## Suggested Fix

FR-010 Pass 3 or `amvacuumcleanup` should update the metadata entry_point if the current entry
point was deleted. Choose a new entry point from the remaining live nodes at the highest level
using the same `choose_entry_point` logic from build.

This is a future concern — documenting it now so FR-010 implementation accounts for it.
