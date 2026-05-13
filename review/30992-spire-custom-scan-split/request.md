# Review Request: SPIRE CustomScan Split

Branch: `task-30-spire`
Task row: Phase 12b.3 structural split
Checkpoint scope: layout-only split, no intended behavior change

## Summary

This checkpoint converts `src/am/ec_spire/custom_scan.rs` into
`src/am/ec_spire/custom_scan/mod.rs` plus concern-oriented included files.
The split keeps the current module name and textual scope, so existing
`custom_scan::...` use paths and callback symbol visibility are preserved.

## Layout Changes

- `custom_scan/mod.rs`: imports, constants, state structs, registration, status.
- `custom_scan/planner.rs`: eligibility checks, path hook, path construction.
- `custom_scan/cost_helpers.rs`: CustomScan cost estimate helpers.
- `custom_scan/plan_private.rs`: plan-private decode/encode helpers.
- `custom_scan/begin_exec.rs`: create/begin/exec/end/rescan/access/recheck callbacks.
- `custom_scan/tuple_payload.rs`: JSON and typed tuple payload slot storage.
- `custom_scan/dml.rs`: DML CustomScan metadata and execution helpers.
- `custom_scan/tests.rs`: relocated former inline `#[cfg(test)]` block.

The RemoteScan test-fill rows remain open: Begin/End/ReScan/read-cancel,
ExplainCustomScan JSON, and empty-remote-result fixtures were not implemented
in this layout-only checkpoint.

## Validation

Artifacts are in `review/30992-spire-custom-scan-split/artifacts/`.

- `cargo check --no-default-features --features pg18`: pass.
- `cargo fmt --check`: pass, with existing stable-rustfmt config warnings.
- `git diff --check -- ...`: pass.
- `cargo test --no-default-features --features pg18 custom_scan`: pass,
  13 passed / 0 failed / 1699 filtered out.
- Per-file line-count sanity: largest new file is `dml.rs` at 746 lines.

## Review Focus

- Confirm the split preserves the CustomScan callback wiring and public
  re-exports.
- Confirm the tracker correctly marks only structural rows complete and leaves
  RemoteScan test-fill rows open.
- Confirm the file boundaries are workable before the behavior-test fill work
  lands.

