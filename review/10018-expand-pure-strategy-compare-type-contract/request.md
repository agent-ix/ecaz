# Review Request: Expand Pure Strategy CompareType Contract

Scope:
- `src/am/cost.rs`
- `spec/functional/FR-023-strategy-translation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Expanded `PlannerCompareType` in `src/am/cost.rs` to model the broader generic CompareType domain
  (`COMPARE_LE`, `COMPARE_EQ`, `COMPARE_GE`, `COMPARE_GT`, `COMPARE_NE`,
  `COMPARE_OVERLAP`, `COMPARE_CONTAINED_BY`) rather than only `COMPARE_LT` and
  `COMPARE_INVALID`.
- Kept the intended tqvector mapping strict: only `COMPARE_LT` maps back to strategy 1, while all
  other compare types now explicitly return `InvalidStrategy` in pure code.
- Added test coverage for the broader reverse-mapping behavior and updated FR-023 / the test matrix
  / Task 11 notes to record that this callback contract is now modeled more completely in pure
  scaffolding.

Review focus:
- Whether the expanded `PlannerCompareType` enum is the right pure representation before PG18
  callback bindings exist
- Whether keeping all non-`LT` compare types mapped to `InvalidStrategy` is the right strict
  behavior for tqvector’s single-strategy ordering semantics
- Whether this addresses the earlier reviewer concern that strategy translation should live as pure
  callback scaffolding in `am/cost.rs`, not primarily in explain-facing surfaces

Questions to answer:
- Is there any additional CompareType case that should be modeled now for the future PG18 binding?
- Is `PlannerCompareType` still the right name, or should it be brought closer to PostgreSQL’s
  `CompareType` terminology before live bindings exist?
- Does this make the pure strategy-translation seam complete enough to defer further FR-023 work
  until the actual PG18 callback registration step?
