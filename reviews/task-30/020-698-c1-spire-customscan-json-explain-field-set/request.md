# Review Request: SPIRE CustomScan JSON EXPLAIN field-set coverage

- coder: coder1
- date: 2026-05-14
- code commit: f21c0571 `Pin CustomScan JSON explain field set`
- topic: SPIRE phase 12c.10.e CustomScan EXPLAIN JSON field-set contract

## Scope

This slice tightens the existing loopback CustomScan JSON EXPLAIN test by
pinning the SPIRE-specific field set emitted by `ec_spire_explain_custom_scan`.

Changed file:

- `src/tests/custom_scan.rs`

## What Changed

In `test_ec_spire_customscan_returns_loopback_remote_tuple_payload`, the test
now extracts the `Custom Scan` JSON plan node and asserts the SPIRE-specific
field set is exactly:

- `node`
- `remote_fanout`
- `tuple_transport_status`
- `nprobe`
- `rerank_width`

The assertion is paired with a code comment documenting that these fields are
the extension-owned EXPLAIN contract layered on top of PostgreSQL's standard
Custom Scan JSON keys. Existing assertions still pin representative values and
ANALYZE counters.

## Test File Size Discipline

The touched test file remains below the 2500-line target:

```text
1475 src/tests/custom_scan.rs
```

No new fixture file was needed because this is a small assertion on the
canonical loopback CustomScan JSON plan.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_returns_loopback_remote_tuple_payload --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether pinning the extension-owned field set is sufficient for
12c.10.e, or whether reviewers want a follow-up that snapshots all PostgreSQL
standard JSON keys on the Custom Scan node as well.
