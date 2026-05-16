# Review Request: SPIRE DML Baserel Extraction Followups

## Scope

Code commit: `6efe8f84c1c6286b69e336e9536211880a971d03`

This packet folds 30874 review follow-ups into the DML frontdoor baserel
extraction helper.

Changes:

- Documents why `dml_frontdoor_pk_predicate_from_baserestrictinfo(...)` exists
  alongside the analyzed-Query `jointree.quals` path.
- Converts planner relid overflow into an explicit extraction error instead of
  silently returning `None`.
- Keeps unsupported baserel shapes as `None` so internal SPI queries used by the
  PK-select primitive do not get hijacked by the DML CustomScan path.
- Keeps actual extraction errors fail-closed at the CustomScan candidate
  boundary.
- Updates the Phase 11 task file with packet `30875`.

This packet is still PK SELECT hardening only. UPDATE and DELETE routing remain
future slices.

## Validation

- `cargo test dml_frontdoor --lib`
  - `24 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm unsupported internal primitive SELECTs stay outside the DML CustomScan
   candidate path.
2. Confirm true baserel extraction errors still fail closed.
3. Confirm the RestrictInfo-vs-jointree comment captures the intended boundary.
