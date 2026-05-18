# SPIRE Multi-Row Insert Epoch Coverage

## Checkpoint

- Code commit: `7b9e243e`
  (`Cover SPIRE multi-row insert epochs`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up coverage for multi-row post-build inserts

## Summary

This checkpoint closes the literal multi-row insert coverage gap from the
delta-epoch review feedback:

- Added a PG18 regression test that builds a populated `ec_spire` index, then
  inserts five rows with one multi-row SQL `INSERT`.
- The test records the current Phase 1 behavior that PostgreSQL calls
  `aminsert` once per row: active epoch advances from build epoch `1` to
  epoch `6`.
- The test verifies allocator progression after the five delta objects:
  `next_pid = 9` and `next_local_vec_seq = 8`.
- The test verifies active snapshot diagnostics report five delta objects and
  five delta assignments.
- The test verifies an ordered scan over the fixture returns all five
  post-build inserted rows.
- Updated the Task 30 plan to record the multi-row epoch progression coverage.

This is coverage only. It documents the current one-epoch-per-row insert
foundation; it does not implement insert batching or change the delta publish
path.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_multi_row_epoch_progression --no-default-features --features pg18 -- --nocapture`
  - First run exposed an overly strict test expectation: a seed row can outrank
    a nearby inserted row for inner-product `LIMIT 1`.
  - After correcting the assertion to the intended visibility contract:
    `1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `229 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Insert batching remains an explicit follow-up; this test intentionally
  preserves the current one-published-epoch-per-`aminsert` behavior so future
  batching work has a visible contract to update.
