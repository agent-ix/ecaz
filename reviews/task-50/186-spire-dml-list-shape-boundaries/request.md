# Review Request: SPIRE DML List Shape Boundaries

## Scope

Reviews code commit `e767f92b8d097f1217364093be6a2dec227ea5e0`.

This slice consolidates SPIRE DML frontdoor planner list reads:

- Adds a single `DmlFrontdoorFromShape` classifier for jointree `fromlist`
  shape, shared by SELECT single-range-table detection and UPDATE/DELETE
  extra-FROM detection.
- Folds range-table lookup into one audited block that borrows the selected
  `RangeTblEntry` only long enough to copy relation metadata.
- Folds target-list traversal and `TargetEntry` borrowing into one audited
  block that copies target names before returning.

The change keeps the planner-owned pointer reads local to the active callback
and avoids introducing broad safe wrappers over raw PostgreSQL memory.

## Unsafe Movement

- Previous packet 185 ledger: `1796` direct unsafe rows under `src/`
- Packet 186 ledger: `1793` direct unsafe rows under `src/`
- Net reduction: `3`
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `48 -> 45` direct unsafe rows

## Validation

Artifacts are under `artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with existing `src/am/mod.rs` unused import warnings.
- `cargo-check-pg18-pg-test.log`: `cargo check --all-targets --no-default-features --features pg18,pg_test` passed with existing Hadamard test-helper dead-code warnings.
- `cargo-test-dml-frontdoor-predicate-pg18-no-run.log`: targeted DML frontdoor predicate test binary build passed.
- `cargo-pgrx-test-dml-predicate-edge-shapes-pg18-blocked.log`: targeted PG18 pgrx run was blocked before the test body by the existing local `BufferBlocks` symbol lookup failure.
- `rustfmt-dml-frontdoor-check.log`: touched-file rustfmt check passed; stable rustfmt emitted the known unstable option warnings.
- `git-diff-check.log`: `git diff --check HEAD~1..HEAD` passed.
- `unsafe-block-count.log`: records remaining direct unsafe rows in `src/am/ec_spire/dml_frontdoor/mod.rs`.
- `unsafe-ledger-generate.log`: regenerated Task 50 ledger with `1793` rows.
- `unsafe-ledger-check.log`: ledger covers current `src/` unsafe rows.

