# SPIRE Insert Error Path Coverage

## Checkpoint

- Code commit: `f77746ec`
  (`Cover SPIRE insert dimension mismatch`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up coverage for post-build insert error paths

## Summary

This checkpoint closes one post-build insert coverage gap from the delta-epoch
review feedback:

- Added a PG18 regression test that builds an `ec_spire` index over 2D
  `ecvector` rows, then attempts a post-build insert with a 3D vector.
- The test verifies the `aminsert` callback reports the public wrapper
  contract: `ec_spire aminsert failed: ec_spire vector dimensions mismatch`.
- Updated the Task 30 plan to record focused coverage for post-build
  dimension-mismatch insert failures.

This is coverage only. It does not change insert routing, delta epoch
publication, object formats, scan behavior, vacuum behavior, or diagnostics
SQL shape.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_rejects_dimension_mismatch --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out`
  - Captured expected error:
    `ec_spire aminsert failed: ec_spire vector dimensions mismatch: got 3, expected 2`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `228 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Additional insert-path work remains deferred, including true insert batching
  and concurrent insert validation.
