# Review Request: SPIRE DML CustomScan DELETE Executor

## Scope

Code commit: `715b35dfea33a8e0492c067e0c54a34a2c23e1f8`

This packet wires the ADR-069 DELETE executor branch through the DML
`EcSpireDistributedScan` plan-tree replacement.

Changes:

- Routes `DmlDeleteTuplePayload` execution to a real CustomScan access branch
  instead of the prior executor guard.
- Builds the typed DML primitive invocation from runtime state and calls
  `ec_spire_prepare_coordinator_delete_tuple_payload(...)` once.
- Increments `estate->es_processed` from the primitive's
  `remote_deleted_count` and returns no tuple.
- Extends the plan-tree replacement fixture to prove transparent DELETE:
  `ROW_COUNT = 1`, the local heap row is gone, and the placement row is gone.

This closes the UPDATE/DELETE executor-dispatch pair. The ADR-069
documentation cleanup for plan-tree replacement limitations remains open.

## Validation

- `cargo test test_ec_spire_dml_plan_tree_replace_scaffold --lib`
  - `1 passed; 0 failed; 0 ignored; 1682 filtered out`
  - artifact: `artifacts/cargo-test-dml-plan-tree-replace-delete.log`
- `cargo test custom_scan --lib`
  - `14 passed; 0 failed; 0 ignored; 1669 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check 715b35df^ 715b35df -- src/am/ec_spire/custom_scan.rs src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm DELETE execution correctly maps DML CustomScan runtime state to the
   existing coordinator delete primitive.
2. Confirm using `remote_deleted_count` for `estate->es_processed` is correct
   for both local delete and future remote prepared-delete paths.
3. Confirm the fixture covers the user-visible DELETE contract for the current
   local-placement path without expanding scope into remote 2PC visibility.
