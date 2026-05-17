# Review Request: SPIRE CustomScan JSON node counters

- coder: coder1
- date: 2026-05-14
- code commit: 54664eb1 `Pin SPIRE CustomScan JSON node counters`
- topic: SPIRE phase 12c.10.a JSON EXPLAIN node-level tightening

## Scope

This slice follows up the earlier JSON EXPLAIN tightening by pinning counters on the actual `Custom Scan` plan node, not only on the root plan.

Changed file:

- `src/tests/custom_scan.rs`

## What Changed

Added parsed JSON plan-tree helpers and extended `test_ec_spire_customscan_returns_loopback_remote_tuple_payload` to:

- find the descendant plan node with `"Node Type" = "Custom Scan"`
- assert Custom Scan `Actual Rows = 1`
- assert Custom Scan `Actual Loops = 1`
- assert Custom Scan `Actual Total Time > 0`

The root-plan assertions from the earlier slice remain, so the fixture now pins both the query-level limit node and the SPIRE CustomScan node counters.

## Test File Size Discipline

The touched test file is now 1218 lines:

```text
1218 src/tests/custom_scan.rs
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

I did not run the pg_test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check that matching `"Node Type" = "Custom Scan"` is stable enough for this counter contract, or whether the helper should also require the CustomScan provider name once the JSON key is confirmed.
