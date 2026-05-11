# Review Request: SPIRE CustomScan Payload Scalar Gate

Follow-up slice for the `30814` CustomScan tuple-payload review. This closes
the immediate P1 array/non-scalar payload risk on the JSON bridge and removes
the per-row `fmgr_info` setup cost while the final typed-tuple transport remains
deferred.

## Scope

- Rejects projected tuple-payload columns whose PostgreSQL type is an array or
  composite row type during `BeginCustomScan`, before remote dispatch.
- Rejects JSON array/object payload values before calling PostgreSQL type input
  functions, so unsupported values fail with an EcSpireDistributedScan error
  instead of a confusing typinput parse error.
- Caches per-attribute `(FmgrInfo, typioparam, typmod)` input conversion state
  on `SpireCustomScanExecState` instead of rebuilding it for every row and
  column.
- Keeps the scalar JSON payload bridge for currently covered scalar projected
  columns.
- Adds PG18 coverage for `SELECT tags text[] ... ORDER BY embedding ... LIMIT`
  failing closed on the CustomScan tuple-payload path.
- Updates the CustomScan status next-step and Phase 11 tracker now that packet
  `30815` supplied the first loopback read fixture.

## Validation

- `cargo test customscan --lib`
  - 8 passed, including the new array-projection fail-closed fixture.
- `cargo test tuple_payload --lib`
  - 7 passed, including the same array-projection fail-closed fixture plus
    tuple-payload endpoint coverage.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.

## Review Focus

- Check whether the fail-closed type gate is the right interim contract before
  a typed tuple/composite payload transport replaces the JSON bridge.
- Check that `SpireCustomScanPayloadAttrInput` lifetime is valid for scan-state
  storage and that `InputFunctionCall` receives the cached mutable `FmgrInfo`.
- Check that the projected-column validation still only rejects requested
  payload columns, not non-projected wide/array columns.
- Remaining open items from the 30814 review: degraded-mode missing-payload
  skip accounting, remote-side projection pushdown, and eventual removal of the
  JSON bridge/`serde_json` dependency when typed transport lands.

## Artifacts

- `review/30816-spire-customscan-payload-scalar-gate/artifacts/manifest.md`
- `review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-test-customscan-lib.log`
- `review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-test-tuple-payload-lib.log`
- `review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-fmt-check.log`
- `review/30816-spire-customscan-payload-scalar-gate/artifacts/git-diff-check.log`
