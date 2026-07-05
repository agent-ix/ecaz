# Review Request: SPIRE DML Plan-Tree Replacement Scaffold

## Scope

Code commit: `0e63d2c34f2c348bb9dd63feecb3addfbf7684e5`

This packet adds the reviewer-recommended DML planner-hook scaffold for
ADR-069 UPDATE/DELETE routing.

Changes:

- Captures the supported UPDATE/DELETE primitive expression before
  `standard_planner`, then replaces `PlannedStmt.planTree` afterward with a
  top-level `CustomScan` plan.
- Leaves PK SELECT on the existing baserel CustomPath path.
- Adds `custom_scan_dml_replacement_plan(...)` to build a no-scanrelid
  `EcSpireDistributedScan` plan carrying the copied PK expression and DML
  plan-private metadata.
- Keeps UPDATE/DELETE execution fail-closed with the existing
  `EcSpireDistributedScan DML UPDATE/DELETE executor path is not wired yet`
  guard.
- Avoids relation-backed tuple-payload slot initialization for UPDATE/DELETE
  scaffold plans, because the top-level DML CustomScan has `scanrelid = 0`.
- Updates hook status to report `plan_rewrite_enabled = true` and records
  `last_hook_action = plan_tree_replaced_customscan` after replacement.
- Adds focused PG18 coverage proving UPDATE/DELETE EXPLAIN planning reaches the
  replacement action and actual UPDATE execution still fails closed.

## Validation

- `cargo test test_ec_spire_dml_plan_tree_replace_scaffold --lib`
  - `1 passed; 0 failed; 0 ignored; 1681 filtered out`
  - artifact: `artifacts/cargo-test-dml-plan-tree-replace-scaffold.log`
- `cargo test custom_scan --lib`
  - `13 passed; 0 failed; 0 ignored; 1669 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check 0e63d2c3^ 0e63d2c3 -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the hook captures primitive expression state before
   `standard_planner` and replaces `PlannedStmt.planTree` only after the
   standard/chained planner returns.
2. Confirm only UPDATE/DELETE use this top-level plan replacement; PK SELECT
   remains on the existing CustomPath route.
3. Confirm executor behavior is intentionally fail-closed until the follow-up
   UPDATE and DELETE dispatch packets.
