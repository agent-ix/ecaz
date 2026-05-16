# Review Request: SPIRE DML CustomScan Mode Plumbing

## Scope

Code commit: `cc89b889d10b9c613af165ef41d0575439a2db26`

This packet prepares `EcSpireDistributedScan` for transparent UPDATE and DELETE
rewrite slices without enabling those paths yet.

Changes:

- Adds distinct CustomScan private mode identifiers for DML UPDATE and DELETE,
  while preserving the existing PK SELECT mode value.
- Refactors DML plan construction from PK-SELECT-only to a DML-neutral helper
  that validates the typed frontdoor primitive mode against the requested
  CustomScan plan mode.
- Re-exports the generic baserel primitive-plan expression helper through the
  `ec_spire` module boundary so CustomScan planning can use the shared
  frontdoor extraction path.
- Initializes common DML PK expression state for UPDATE/DELETE plan modes, but
  keeps their executor branch explicitly unwired with a planner-internal error
  if a future path creates them too early.
- Adds a unit test covering DML frontdoor mode to CustomScan plan-private mode
  mapping.

UPDATE/DELETE path generation remains disabled in this packet; the only live
DML CustomScan path remains PK SELECT.

## Validation

- `cargo test custom_scan --lib`
  - `7 passed; 0 failed; 0 ignored; 1667 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/mod.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm this is behavior-preserving for the live PK SELECT DML CustomScan
   path.
2. Confirm UPDATE/DELETE mode identifiers and mapping are coherent with
   `SpireDmlFrontdoorCustomScanMode`.
3. Confirm the explicit UPDATE/DELETE executor error is acceptable until the
   next slice wires those branches to coordinator primitives.
