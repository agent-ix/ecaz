# Review Request: Align Pure Callback Helper Names With PG18

Scope:
- `src/am/cost.rs`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/functional/FR-023-strategy-translation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Renamed the pure tree-height and strategy-translation helpers in `src/am/cost.rs` to
  PG18-callback-aligned names: `amgettreeheight_callback_value(...)`,
  `amtranslatestrategy_callback(...)`, and `amtranslatecmptype_callback(...)`.
- Kept the behavior identical: tree height still returns the metadata `max_level`, strategy 1 still
  maps to `COMPARE_LT`, and only `COMPARE_LT` reverse-maps back to strategy 1.
- Updated unit coverage and the FR-020 / FR-023 / test-matrix / Task 11 wording so the code and
  docs now use the same callback vocabulary.

Review focus:
- Whether moving the pure helper names closer to the eventual PG18 callback surface is the right
  cleanup now, before real `IndexAmRoutine` bindings exist
- Whether the renamed helpers make the D1 seam clearer without implying that any live PG18 binding
  has landed
- Whether keeping the same strict strategy semantics under the callback-aligned names is the right
  planner contract for tqvector’s single ordering operator

Questions to answer:
- Are these callback-aligned names the right long-lived API for the pure planner seam, or should
  they stay more abstract until PG18 is actually compiled in?
- Does this resolve the remaining “two vocabularies for one contract” issue in `am/cost.rs` cleanly
  enough to stop touching FR-020/FR-023 D1 scaffolding?
- Is there any other pure planner callback seam still obviously misnamed before later PG18 binding
  work starts?
