# Review Request: SPIRE DML PK SELECT CustomScan

## Scope

Code commits:

- `ca4fa198fd34509f34bc6d2b8a73fda28ae5c907` — wires PK SELECT through
  `EcSpireDistributedScan`.
- `adcca43be8a86da8ed0be137a073e859cb1425aa` — addresses pre-checkpoint
  reviewer feedback by replacing the planner-hook SPI placement probe with a
  direct `ec_spire_placement` relation scan.

This packet wires the first transparent ADR-069 DML read path into
`EcSpireDistributedScan`: PK-keyed `SELECT` can now plan as a CustomScan when
the target ec_spire index has placement-directory rows.

Changes:

- Adds explicit CustomScan plan modes so the existing vector `ORDER BY ... LIMIT`
  path and the new DML PK SELECT path can coexist behind the same provider.
- Adds a DML PK SELECT CustomPath candidate for baserel `SELECT` plans with a
  single bigint-compatible primary-key equality qual and at least one
  `ec_spire_placement` row for the target index.
- Copies the matched PK value expression into `custom_exprs` and evaluates it in
  `BeginCustomScan`, preserving parameter support for later prepared-statement
  coverage.
- Executes the coordinator PK-select tuple-payload primitive from the CustomScan
  executor and stores the returned JSON payload into the scan slot.
- Converts `RestrictInfo` lists to executable plan quals with
  `extract_actual_clauses(...)`; this fixes filtered CustomScan plans generally,
  not only DML mode.
- Adds a PG18 fixture proving `SELECT id, title ... WHERE id = 1111` plans as
  `Custom Scan (EcSpireDistributedScan)` and returns the row through the
  coordinator PK-select primitive.
- Addresses reviewer P1 feedback on planner-hook SPI re-entrance risk by
  probing placement existence through `table_open` / `table_beginscan_catalog`
  instead of SPI.
- Updates the Phase 11 task file with packet `30873`.

This packet does not implement UPDATE or DELETE CustomScan/ModifyTable routing.
It also keeps the DML CustomScan path placement-gated so classifier-only tables
without `ec_spire_placement` rows continue to pass through the prior planner
behavior.

The pre-checkpoint review also flagged the remaining duplicated PK extraction
between `custom_scan.rs` and `dml_frontdoor.rs`. This packet keeps the
baserestrictinfo-based extractor local because `set_rel_pathlist_hook` sees the
planner-normalized `RestrictInfo` list while the existing 30872 helper consumes
the analyzed query's `jointree.quals`. A follow-up should move that
baserestrictinfo-aware extraction into the DML frontdoor module before adding
UPDATE/DELETE routing.

## Validation

- `cargo test dml_frontdoor --lib`
  - `24 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check ca4fa198fd34509f34bc6d2b8a73fda28ae5c907^ HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the DML CustomScan mode split preserves the existing vector
   `ORDER BY ... LIMIT` CustomScan path.
2. Confirm the PK SELECT path is appropriately placement-gated without using
   SPI from inside `set_rel_pathlist_hook`.
3. Confirm `extract_actual_clauses(...)` is the right conversion for
   `PlanCustomPath` quals before assigning `scan.plan.qual`.
4. Confirm invoking `ec_spire_forward_coordinator_select_tuple_payload(...)`
   from the CustomScan executor is acceptable for this first transparent
   PK-read slice.
