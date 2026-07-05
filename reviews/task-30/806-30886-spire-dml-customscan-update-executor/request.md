# Review Request: SPIRE DML CustomScan UPDATE Executor

## Scope

Code commit: `66e652290f32760323e48940cbbdddfc84cc0d52`

This packet wires the first transparent ADR-069 UPDATE executor path through
the DML `EcSpireDistributedScan` plan-tree replacement.

Changes:

- Carries UPDATE target value expressions through `CustomScan.custom_exprs`
  alongside the PK expression.
- Adds plan-private PK-column metadata decoding so UPDATE/DELETE top-level
  replacement plans no longer depend on the scan relation context at execution.
- Implements the UPDATE executor branch:
  - builds a typed DML primitive invocation from runtime state;
  - converts constant and parameter SET values into the JSONB row-payload
    contract expected by `ec_spire_forward_coordinator_update_tuple_payload`;
  - calls the coordinator update primitive once;
  - increments `estate->es_processed` from `remote_updated_count`;
  - returns no tuple.
- Keeps row-dependent SET expressions fail-closed for v1 with an explicit
  error message.
- Makes the PK SELECT CustomPath candidate probe decline non-PK-SELECT DML
  shapes instead of treating UPDATE/DELETE mode mismatch as fatal. UPDATE and
  DELETE are owned by the planner-hook plan-tree replacement path.

DELETE executor wiring remains intentionally open.

## Validation

- `cargo test test_ec_spire_dml_plan_tree_replace_scaffold --lib`
  - `1 passed; 0 failed; 0 ignored; 1682 filtered out`
  - artifact: `artifacts/cargo-test-dml-plan-tree-replace-update.log`
- `cargo test custom_scan --lib`
  - `14 passed; 0 failed; 0 ignored; 1669 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check 66e65229^ 66e65229 -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm UPDATE execution correctly maps `CustomScan` state to the existing
   coordinator update primitive without reintroducing a coordinator heap update
   path.
2. Confirm `estate->es_processed` is incremented in the right place for the
   top-level plan-tree replacement approach.
3. Confirm the constant/parameter-only SET-value boundary is the right v1
   fail-closed scope until row-dependent expression evaluation is designed.
4. Confirm the PK SELECT candidate probe should silently decline UPDATE/DELETE
   mode mismatch while the DML planner hook owns those operations.
