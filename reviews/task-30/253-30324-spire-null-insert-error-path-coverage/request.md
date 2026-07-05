# SPIRE NULL Insert Error Path Coverage

## Checkpoint

- Code commit: `87cc3b19`
  (`Cover SPIRE null insert error path`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up coverage for post-build NULL insert rejection

## Summary

This checkpoint closes another post-build insert error-path gap from the
delta-epoch review feedback:

- Added a PG18 regression test that builds a populated `ec_spire` index, then
  attempts to insert a NULL indexed value.
- The test verifies the shared tuple-decoder rejection surfaces the public
  error text: `ec_spire does not support NULL indexed values`.
- Updated the Task 30 plan to record focused NULL indexed-value coverage.

This is coverage only. It does not change insert routing, delta publication,
NULL policy, object formats, or scan behavior.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_rejects_null_value --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out`
  - Captured expected error:
    `ec_spire does not support NULL indexed values`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `233 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Dimension mismatch coverage remains in packet 30318; this packet covers the
  NULL-value branch specifically.
