# Review Request: SPIRE Tuple Transport Retired Live Fixture

agent: coder1
date: 2026-05-14
code_commit: 8067bc1c
task: SPIRE task 12c.2.b

## Summary

Adds live CustomScan coverage for the updated 12c.2.b
`tuple_transport_retired` rows. Earlier packet `717` covered the executor
state matrix but explicitly left the production path partial until a live
remote advertisement fixture existed.

This slice uses the current broken-down task file as source of truth and marks
only the two explicit 12c.2.b rows complete.

## Changes

- Added `src/tests/custom_scan_tuple_transport.rs` as a new small CustomScan
  test slice instead of extending the existing execution/concurrency files.
- Included the new slice from `src/tests/mod.rs`.
- Added `test_ec_spire_customscan_tuple_transport_retired_live_sql`:
  - builds matched coordinator and loopback remote `ec_spire` indexes,
  - shadows `ec_spire_remote_search_endpoint_identity(oid)` in a loopback
    search-path schema,
  - preserves the valid endpoint identity envelope from the real public
    endpoint identity function,
  - advertises only `json_tuple_payload_v1`,
  - asserts strict CustomScan execution fails closed with
    `tuple_transport_retired`,
  - asserts degraded CustomScan execution skips the retired remote,
  - asserts `ec_spire_remote_search_degraded_skip_report` exposes a hint that
    names `pg_binary_attr_v1`.
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` 12c.2.b with the
  new fixture as evidence.

## File-Size Discipline

- `src/tests/custom_scan_tuple_transport.rs`: 154 lines.
- `src/tests/custom_scan_execution.rs`: 348 lines.
- `src/tests/custom_scan_concurrency.rs`: 572 lines.
- `src/tests/custom_scan.rs`: 1353 lines.

The umbrella `src/tests/mod.rs` was already over the 2500-line target and this
slice adds only the include line. The new coverage itself lives in a dedicated
small file to avoid rebuilding a large mixed-purpose CustomScan test file.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/tests/mod.rs src/tests/custom_scan_tuple_transport.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_tuple_transport_retired_live_sql --no-run`

Attempted PG18 runtime:

- `cargo pgrx test pg18 test_ec_spire_customscan_tuple_transport_retired_live_sql`

Result: failed before the test body executed with the existing local loader
issue:

```text
undefined symbol: pg_re_throw
```

## Review Focus

- Does the search-path endpoint identity shadowing exercise the intended live
  production tuple transport negotiation path without over-coupling to unrelated
  endpoint identity fields?
- Is it acceptable to close both 12c.2.b rows with one fixture that covers
  strict fail-closed behavior and the degraded skip-report hint?
- Is the new `custom_scan_tuple_transport.rs` boundary the right place for
  future tuple transport-specific CustomScan fixtures?
