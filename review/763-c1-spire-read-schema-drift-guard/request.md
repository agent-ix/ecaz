# Review Request: SPIRE READ Schema Drift Guard

## Summary

Coder: `coder1`
Topic: `763-c1-spire-read-schema-drift-guard`
Code commit: `9d3c7b9cab65162b0ac2a4437d0d116b75c2ed4e`
Date: `2026-05-15`

This packet supersedes the earlier 12c.4 deferral packets (`758`/`759`) by
landing the narrow production guard and focused test fixture instead of
carrying the gap into Phase 13.

## Changes

- Added READ-path schema fingerprint validation before libpq READ dispatch.
  The connection plan compares descriptor-bound coordinator and remote shape
  fingerprints against the current coordinator and remote fingerprints.
- Strict mode fails closed with `schema_drift` and side-specific detail:
  coordinator-only, remote-only, or both-sides drift.
- Degraded mode marks the remote as pre-dispatch `schema_drift`, blocks the
  pipeline, and lets existing degraded skip/report surfaces expose the skip.
- Added `src/tests/custom_scan_schema_drift.rs` so the new fixture does not
  bloat the existing split CustomScan files. The file is 204 lines.
- Updated the Phase 12c tracker and Phase 13 gate so 12c.4 is recorded as live
  coverage rather than a deferral.

## Files

- `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`
- `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs`
- `src/am/ec_spire/coordinator/remote_candidates/contracts.rs`
- `src/tests/custom_scan_schema_drift.rs`
- `src/tests/mod.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`
- `plan/tasks/task30-phase13-spire-aws-verification.md`

## Validation

- `cargo fmt --check` passed; rustfmt still emits the existing stable-channel
  warnings for unstable import-group settings.
- `cargo test --no-default-features --features "pg18 pg_test" test_ec_spire_customscan_read_schema_drift_variants_sql --no-run`
  passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_read_schema_drift_variants_sql`
  was attempted but did not enter the test body; the local harness failed with
  the existing loader error `undefined symbol: pg_re_throw`.
- `git diff --check` passed.

Logs are under `review/763-c1-spire-read-schema-drift-guard/artifacts/`.

## Review Needs

Please verify:

- the READ guard is placed early enough to prevent schema-drifted libpq READ
  dispatch in strict mode;
- degraded mode's `schema_drift` block/skip behavior is the right policy;
- the side-specific messages satisfy 12c.4.a/b/c;
- the new fixture file and tracker updates are sufficient to close 12c.4 and
  remove the Phase 13 deferral gate.
