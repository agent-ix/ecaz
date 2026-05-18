# Review Request: Strategy Translation Scaffolding In Explain Snapshot

Scope:
- `src/am/cost.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-006-sql-operators.md`
- `spec/functional/FR-023-strategy-translation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added pure planner-side strategy-translation helpers in `src/am/cost.rs` that encode the
  intended PG18 mapping for tqhnsw ordering: strategy `1` maps to `COMPARE_LT`, invalid inputs map
  to `COMPARE_INVALID`, and reverse translation from `COMPARE_LT` maps back to strategy `1`.
- Kept the callbacks explicitly unavailable by reporting `pg18_callback_ready = false` in the pure
  strategy-translation snapshot, since this repository still does not have PG18 toolchain support
  or `IndexAmRoutine` PG18 fields.
- Extended `tqhnsw_index_explain_snapshot(regclass)` to expose `ordering_strategy`,
  `ordering_compare_type`, and `pg18_strategy_translation_ready`, so explain/planner tooling can
  inspect the intended ordering contract without confusing it for live planner wiring.
- Added unit and pg coverage for the pure mapping and the explain-snapshot contract, and updated
  FR-006 / FR-023 / test matrix / Task 11 progress notes to record this as D1 scaffolding.

Review focus:
- Whether surfacing the intended strategy translation through the explain snapshot is the right
  planner-facing seam before PG18 callback registration exists
- Whether the pure mapping helper shape in `src/am/cost.rs` is a good long-lived precursor to the
  eventual `amtranslatestrategy` / `amtranslatecmptype` callback implementation
- Whether exposing `pg18_strategy_translation_ready = false` is explicit enough to prevent
  accidental overstatement of planner readiness

Questions to answer:
- Is `tqhnsw_index_explain_snapshot(...)` the right place to expose the intended ordering mapping,
  or should this remain purely internal until PG18 support lands?
- Is the local `PlannerCompareType` staging enum a sensible compatibility seam, or should the code
  wait for real PG18 bindings instead of carrying a repo-local representation?
- Does this make the FR-023 near-term versus target-state story clearer without adding too much
  planner-facing surface area too early?
