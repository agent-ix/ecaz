# Review Request: SPIRE CustomScan selected PID payload coverage

- coder: coder1
- date: 2026-05-14
- code commit: a9010931 `Cover SPIRE CustomScan selected PID payloads`
- topic: SPIRE phase 12c.7.b selected-PID round-trip assertion

## Scope

This slice adds focused CustomScan tuple payload coverage for the selected-PID
remote scan contract.

Changed file:

- `src/tests/custom_scan.rs`

## What Changed

Added `test_ec_spire_customscan_selected_pid_payloads`.

The fixture builds matching coordinator and loopback remote indexes over eight
known rows, rewrites all coordinator placements to a loopback remote node, and
captures the active epoch plus the selected coordinator leaf PIDs.

The test asserts:

- coordinator and remote active epochs match
- coordinator and remote selected leaf PIDs match
- one-PID remote tuple payload probes only return that requested PID
- the all-PID remote tuple payload probe returns exactly the selected PID set
- the all-PID payload rows equal the union of the individual per-PID probes
- forced CustomScan `LIMIT 8` output matches the remote PID-to-payload mapping

The SQL-facing CustomScan result does not expose PID directly, so the test pins
the selected-PID mapping through `ec_spire_remote_search_tuple_payload` and then
asserts the CustomScan rows match that selected remote payload set.

## Test File Size Discipline

The touched test file remains below the 2500-line target:

```text
1448 src/tests/custom_scan.rs
```

No new test file was needed for this slice. `custom_scan.rs` still has enough
headroom after the recent split, and the new fixture is grouped with the
existing loopback CustomScan payload coverage.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_selected_pid_payloads --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether the two-step assertion is sufficient for 12c.7.b:

1. prove `ec_spire_remote_search_tuple_payload` returns exactly the selected
   PID set with the expected PID-to-payload mapping
2. prove CustomScan returns the same payload set for `LIMIT 8`

If reviewers need executor-visible PID evidence directly from the CustomScan SQL
result, that will need a follow-up diagnostic surface because the current user
query projection only exposes row payload columns.
