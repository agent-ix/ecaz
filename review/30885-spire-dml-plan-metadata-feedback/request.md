# Review Request: SPIRE DML Plan Metadata Feedback Follow-Up

## Scope

Code commit: `d5c7d66cbf44966da286638fc69dfe309cc29e9b`

This packet folds in focused feedback from 30883 and 30884 before the
UPDATE/DELETE executor wiring packets.

Changes:

- Documents that DML `CustomScan.custom_private` must remain a homogeneous
  `T_List<T_String>` layout, so future metadata slots do not reintroduce the
  mixed `OidList` / `T_String` bug fixed in 30883.
- Adds a PG-backed `copyObject` roundtrip regression for DML plan-private
  metadata.
- Factors DML plan-private column-list decoding so both real plans and the
  roundtrip test use the same decoder.
- Documents that DML plan-tree replacement copies fallback costs only for
  reasonable EXPLAIN output because there is no path competition after
  `PlannedStmt.planTree` replacement.
- Documents that `plan_rewrite_enabled = true` means supported UPDATE/DELETE
  shapes are planned as CustomScan while execution is still gated by per-mode
  executor wiring.

No behavior changes are intended.

## Validation

- `cargo test test_ec_spire_custom_scan_dml_plan_private_copyobject_sql --lib`
  - `1 passed; 0 failed; 0 ignored; 1682 filtered out`
  - artifact: `artifacts/cargo-test-dml-plan-private-copyobject.log`
- `cargo test custom_scan --lib`
  - `14 passed; 0 failed; 0 ignored; 1669 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check d5c7d66c^ d5c7d66c -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/am/ec_spire/mod.rs src/am/mod.rs src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the comments accurately pin the DML plan-private and plan-rewrite
   scaffold contracts without overstating executor readiness.
2. Confirm the `copyObject` regression exercises the same decoder used by real
   DML CustomScan plans.
3. Confirm there is no change to UPDATE/DELETE execution behavior; executor
   dispatch remains the next packet.
