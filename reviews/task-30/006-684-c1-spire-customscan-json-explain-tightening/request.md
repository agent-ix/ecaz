# Review Request: SPIRE CustomScan JSON EXPLAIN tightening

- coder: coder1
- date: 2026-05-14
- code commit: 567eb8de `Tighten SPIRE CustomScan JSON explain coverage`
- topic: SPIRE phase 12c.10.a JSON EXPLAIN assertion tightening

## Scope

This slice tightens the existing CustomScan loopback payload fixture so it validates parsed JSON EXPLAIN fields instead of relying only on substring checks.

Changed file:

- `src/tests/custom_scan.rs`

## What Changed

Added a small `custom_scan_json_explain_root_plan` helper and extended `test_ec_spire_customscan_returns_loopback_remote_tuple_payload` to assert:

- root JSON plan includes `Actual Rows = 1`, matching the query `LIMIT`
- root JSON plan includes `Actual Loops = 1`
- root JSON plan includes a positive `Actual Total Time`

The existing CustomScan-specific JSON substring checks remain in place for the custom `EcSpireDistributedScan` properties.

## Test File Size Discipline

The touched test file is now 1122 lines:

```text
1122 src/tests/custom_scan.rs
```

No large test file was expanded past the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_returns_loopback_remote_tuple_payload --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether pinning the root JSON plan is the right contract for 12c.10.a, or whether the assertion should descend to the `Custom Scan` child node for the actual-row/loop/time counters.
