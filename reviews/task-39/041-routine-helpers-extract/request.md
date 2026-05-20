# Task 39 / 041 — Extract Pure Helpers from ec_diskann/routine.rs

## Goal

Apply the same coverage extraction pattern from packet 040 to
`src/am/ec_diskann/routine.rs`: move the three small pure helpers
into a sibling `routine_helpers.rs` so the careful crate can
`include!` them and exercise every branch under `make coverage`.
`routine.rs` itself is dominated by pgrx-FFI callback code that
requires a live PG18 backend to cover, so this is the only mechanical
slice available without a much larger pgrx scaffold.

## Code Change

`src/am/ec_diskann/routine_helpers.rs` (new):

- `sort_and_dedup_item_pointers(tids)` — pure sort + dedup using
  `insert::cmp_item_pointer_physical`.
- `vacuum_repair_scan_budget(build_list_size, graph_degree_r)` —
  `build_list_size.min(graph_degree_r.max(1))`, clamping repair scans
  to at least one neighbor.
- `sql_scan_result_cap(reloption_top_k, rerank_budget)` — returns the
  rerank budget regardless of the reloption (SQL path can't see the
  `LIMIT`).

All function bodies are byte-for-byte identical to what they replaced
in `routine.rs`.

`src/am/ec_diskann/routine.rs`:

- Replaces the moved definitions with `include!("routine_helpers.rs")`.

`hardening/careful/src/lib.rs` + `hardening/careful/src/diskann_routine_helpers.rs`:

- New careful module that provides a local `insert::cmp_item_pointer_physical`
  shim (the production version is `pub(super)` inside `ec_diskann/insert.rs`
  and visible to the helper through mod.rs scope), then `include!`s
  the production helpers verbatim.
- 3 unit tests cover every branch of every extracted helper, including
  the `graph_degree_r=0` clamp branch in `vacuum_repair_scan_budget`.

`scripts/check_coverage_baseline_complete.sh`:

- Adds `src/am/ec_diskann/routine_helpers.rs` to the critical-paths
  set (now 42 paths).

`fixtures/quality/coverage-baseline.tsv`:

- New row `am/ec_diskann/routine_helpers.rs 100.00`.

## Baseline Net Effect

| File | Before | After |
| --- | ---: | ---: |
| `am/ec_diskann/routine.rs` | 0.00 (1533 lines) | 0.00 (1522 lines) |
| `am/ec_diskann/routine_helpers.rs` | (did not exist) | **100.00** (11 lines) |
| Critical paths tracked | 41 | 42 |

The 11 extracted lines are now exercised under `make coverage`. The
remaining 1522 lines of `routine.rs` are the pgrx-FFI callback surface
(`amhandler`, vacuum callbacks, bulk-insert plumbing, scan-state
construction) that require either a live PG18 backend or a deeper
pgrx-bindings shadow crate. Documented as the upstream blocker in
`docs/hardening.md::## Test Quality`.

## Validation

Artifacts under
`reviews/task-39/041-routine-helpers-extract/artifacts/`:

- `routine-helpers-extract-focused-tests.log`: **511 passed**.
- `coverage/summary.txt` + JSON: `routine_helpers.rs 100.00%`,
  `routine.rs 0.00%`.
- `coverage-delta-check.log`: every baseline row green.
- `coverage-baseline-check.log`: **42 critical paths complete**.
- Production `cargo check --features pg18 --no-default-features`
  is clean (no behavior change; function bodies identical).
