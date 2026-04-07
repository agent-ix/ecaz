# Review Request: Pure Tree-Height Callback Helper

Scope:
- `src/am/cost.rs`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added `metadata_tree_height_callback_value(max_level)` in `src/am/cost.rs` as a pure helper for
  the eventual PG18 `amgettreeheight` callback contract.
- Added unit coverage that verifies the helper returns the exact metadata `max_level` value,
  including the `u8::MAX` edge case.
- Updated FR-020, the test matrix, and Task 11 notes to record this as a pure callback-value seam:
  the integer callback contract is explicit, but the actual PG18 `IndexAmRoutine` binding is still
  pending.

Review focus:
- Whether this helper is the right D1 seam for the eventual `amgettreeheight` callback
- Whether keeping the callback contract as a pure integer-returning helper is the right level of
  abstraction before PG18 bindings exist
- Whether this slice is the right next step after the earlier metadata-fallback cost work, without
  drifting back into low-value SQL scaffolding

Questions to answer:
- Is `metadata_tree_height_callback_value(...)` the right long-lived helper name, or should it
  more directly mirror `amgettreeheight` now?
- Is there any missing pure logic around tree-height callback semantics that should land before the
  eventual PG18 binding work?
- Does this make the boundary between “cost model uses metadata fallback now” and “PG18 callback
  binding later” clear enough for the other agent?
