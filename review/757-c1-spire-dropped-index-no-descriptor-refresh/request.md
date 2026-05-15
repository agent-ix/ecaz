# Review Request: SPIRE Dropped Index No Descriptor Refresh

## Summary

Coder: `coder1`
Topic: `757-c1-spire-dropped-index-no-descriptor-refresh`
Code commit: `f1d3e3fe2ca1cfb84f995c30a848a8c8d4513842`
Date: `2026-05-15`

This checkpoint closes the remaining non-12c.4 tracker row: 12c.3.b's
assertion that the dropped remote index path does not proceed into endpoint
identity / descriptor-refresh work.

It tightens `test_ec_spire_prod_receive_drop_remote_index_before_dispatch`
with explicit strict and degraded assertions:

- `endpoint_identity_query_count = 0` in strict mode;
- `endpoint_identity_query_count = 0` in degraded mode.

The fixture still asserts the dropped index is classified as
`remote_index_unavailable`, while the ready remote remains usable.

## Files

- `src/tests/remote_search/receive_faults.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`

`src/tests/remote_search/receive_faults.rs` is 1652 lines after this
change, below the 2500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_prod_receive_drop_remote_index_before_dispatch --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_prod_receive_drop_remote_index_before_dispatch` failed before test execution with:
  `undefined symbol: pg_re_throw`.

## Review Needs

Please verify that `endpoint_identity_query_count = 0` is the correct
observable proxy for the row's "no descriptor refresh attempted" wording,
given that the Stage E lifecycle matrix still requires remote index
resolution to classify the dropped index as `remote_index_unavailable`.
