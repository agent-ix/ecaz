# Review Request: SPIRE Degraded Payload Cap Counter

## Summary

This checkpoint closes the remaining Phase 12c.2.a degraded `payload_too_large` tracker row.

Changes:

- Tightened `degraded_skip_report_hints_remote_payload_cap` so it now asserts:
  - `degraded_skipped_dispatch_count = 1`
  - `first_degraded_skip_category = remote_payload_too_large`
  - the degraded skip report surfaces `SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT`
- Updated the Phase 12c tracker row to cite that test.

## Review Focus

Please confirm this is acceptable coverage for the degraded payload-cap row. The strict CustomScan large-text fixture remains `test_ec_spire_customscan_large_text_projection_cap_sql`; this slice pins the executor degraded counter and hint for the same failure category.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features degraded_skip_report_hints_remote_payload_cap --no-run` passed.
- `cargo test --features "pg18 pg_test" --no-default-features degraded_skip_report_hints_remote_payload_cap` failed before running the test with the existing loader error: `undefined symbol: pg_re_throw`.

## Files

- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`
