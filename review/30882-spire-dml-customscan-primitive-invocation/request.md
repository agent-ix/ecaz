# Review Request: SPIRE DML CustomScan Primitive Invocation

## Scope

Code commit: `eada9ab73464bdc4967907774717c19d4896f0fc`

This packet adds the executor-side DML primitive invocation boundary for
`EcSpireDistributedScan`.

Changes:

- Builds a `SpireDmlFrontdoorPrimitiveInvocation` from CustomScan runtime state:
  index OID, DML mode, primitive name, PK column/value, updated columns, and
  projected columns.
- Reuses the 30881 metadata guard before constructing the invocation.
- Routes the live PK SELECT CustomScan executor branch through that invocation
  before calling `ec_spire_forward_coordinator_select_tuple_payload(...)`.
- Adds unit coverage for successful PK SELECT invocation construction and
  fail-closed incomplete state.
- Updates the Phase 11 task file with packet `30882`.

This packet does not enable UPDATE/DELETE path generation and keeps their
executor guard in place.

## Validation

- `cargo test custom_scan --lib`
  - `12 passed; 0 failed; 0 ignored; 1668 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the CustomScan-built invocation matches the DML frontdoor primitive
   contract from packet `30870`.
2. Confirm routing PK SELECT through the invocation is behavior-preserving for
   the currently live DML CustomScan path.
3. Confirm UPDATE/DELETE execution remains disabled despite the shared
   invocation boundary.
