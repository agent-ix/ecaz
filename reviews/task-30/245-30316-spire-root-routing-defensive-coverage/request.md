# SPIRE Root Routing Defensive Coverage

## Checkpoint

- Code commit: `6b4df175`
  (`Cover SPIRE root routing malformed manifests`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up coverage for malformed active root-routing manifests

## Summary

This checkpoint addresses the remaining defensive-coverage gap from the root
routing diagnostics review:

- Extracted the root-routing diagnostic row collector so it can consume any
  `SpireObjectReader`, not only the relation-backed object store.
- Added unit coverage with the local object store for an active published
  snapshot whose object manifest has no root object.
- Added unit coverage for an active published snapshot whose object manifest
  has multiple root objects.
- Added a positive local-store row collection test for the extracted helper,
  keeping the existing SQL surface unchanged.
- Updated the Task 30 plan to record the malformed-manifest coverage.

This is defensive test coverage and a small helper extraction. It does not
change the SQL function shape, root/control publication, object formats,
placement semantics, routing behavior, or scan ordering.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib root_routing_snapshot_ --no-default-features --features pg18`
  - `4 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `226 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- The public relation-backed SQL path still returns zero rows for
  `active_epoch = 0`; the new tests cover malformed active published snapshots
  that normal SQL setup cannot construct directly.
