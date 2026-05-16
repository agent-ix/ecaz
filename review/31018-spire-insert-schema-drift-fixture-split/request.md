# Review Request: SPIRE Insert Schema Drift Fixture Split

## Summary

Packet 31018 extends `src/tests/insert.rs` by moving two coordinator
insert schema-drift fixtures out of `src/tests/mod.rs`:

- `test_ec_spire_schema_drift_fails_before_dispatch_sql`
- `test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql`

The move keeps the existing textual-include strategy, so the fixtures
remain inside the `#[pg_schema] mod tests` scope.

One narrow assertion refresh is included: the coordinator-side fixture
now checks the current structured message contract (`schema_drift` and
`coordinator side drifted`), matching the adjacent remote-side fixture's
existing `schema_drift` style. Product code is unchanged.

Code checkpoint: `efb2786c5acae68058539ed07fd4719a65186b56`

## Review Focus

- Confirm the fixture relocation is mechanical apart from the explicit
  assertion refresh.
- Confirm the assertion refresh matches the current schema-drift message
  emitted by the implementation.
- Confirm `tests/insert.rs` remains open in the tracker because
  insert-after-build and source-identity fixtures still remain in
  `src/tests/mod.rs`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_schema_drift_fails_before_dispatch_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql -- --nocapture`
- `rg -n 'fn test_ec_spire_schema_drift_fails_before_dispatch_sql|fn test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql' src/tests/insert.rs src/tests/mod.rs`
- `wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs`
- `git diff --check`

Artifacts and key result lines are recorded in
`review/31018-spire-insert-schema-drift-fixture-split/artifacts/manifest.md`.
