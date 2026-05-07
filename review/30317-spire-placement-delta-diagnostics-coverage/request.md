# SPIRE Placement Delta Diagnostics Coverage

## Checkpoint

- Code commit: `4cad38ac`
  (`Cover SPIRE placement diagnostics deltas`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up coverage for Delta-kind placement diagnostics

## Summary

This checkpoint closes a per-surface coverage gap in the placement diagnostics:

- Added a unit test that builds a real base epoch and a post-build delta epoch
  through the existing delta draft builder.
- The test verifies aggregate snapshot diagnostics count one delta object, one
  delta assignment, and non-zero delta object bytes.
- The test verifies per-store placement diagnostics count the same delta object
  and include its assignment count and bytes in the single-store totals.
- The test keeps the existing byte-bucket invariant explicit:
  `available_object_bytes = routing_object_bytes + leaf_object_bytes +
  delta_object_bytes`.
- Updated the Task 30 plan to record Delta-kind aggregate/per-store
  diagnostic coverage.

This is coverage only. It does not change diagnostic SQL shape, object
formats, placement semantics, delta publication, scan behavior, or cleanup.

## Changed Files

- `src/am/ec_spire/diagnostics.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib diagnostics_count_delta_objects_and_assignments --no-default-features --features pg18`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `227 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Internal-kind and multi-store placement diagnostics remain deferred to
  recursive routing and local multi-store work.
