# Review Request: Task 28 IVF PG18 Planner Callbacks

Status: open
Owner: coder2
Date: 2026-04-25
Branch: `task28-ivf`
Code checkpoint: `9b045fa7485ac599dd1514539033acdd63a41dd8`

## Scope

- Register PG18 `amgettreeheight` for `ec_ivf`, returning zero for the
  partitioned IVF surface.
- Register PG18 `amtranslatestrategy` / `amtranslatecmptype` for `ec_ivf`,
  reusing the existing `<#>` strategy-1 to `COMPARE_LT` mapping.
- Extend `ec_ivf_index_cost_snapshot(regclass)` with planner-callback
  readiness:
  - `resolved_tree_height`
  - `tree_height_source`
  - `pg18_tree_height_callback_ready`
  - `ordering_compare_type`
  - `pg18_strategy_translation_ready`
- Record the Phase 7 planner-callback checkpoint in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/cost.rs`
- `src/am/ec_ivf/routine.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_cost_snapshot_reports_modeled_costs`
- `git diff --check`

No PG17 tests were run for this checkpoint.

## Review Focus

- Whether zero is the correct PG18 `amgettreeheight` value for the IVF
  partitioned-index surface.
- Whether `ec_ivf` should reuse the shared HNSW `<#>` strategy mapping exactly,
  or whether the callback names/helpers should be generalized before broader
  PG18 runtime work.
- Whether the cost snapshot exposes enough callback-readiness state without
  exceeding pgrx's table tuple arity limit.

## Non-Goals

- PG18 ReadStream wiring.
- Shared pgstat aggregation for IVF counters.
- Recall or latency measurement claims.
