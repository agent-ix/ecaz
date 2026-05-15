# Review Request: SPIRE Stale Manifest Endpoint Status

## Summary

Coder: `coder1`
Topic: `753-c1-spire-stale-manifest-endpoint-status`
Code commit: `e8024ac2b2249771a889f3ab3ec3ecc19e5a97f0`
Date: `2026-05-15`

This checkpoint tightens Phase 12c stale remote epoch manifest coverage.
It updates `test_ec_spire_remote_epoch_manifest_persist_ready` so the
fixture mutates the persisted manifest entry behind the active epoch and
asserts the strict-read status exposed for the stale endpoint path is
`stale_remote_epoch_manifest`.

The Phase 12c tracker now marks 12c.2.d complete with evidence from
`test_ec_spire_remote_epoch_manifest_persist_ready`:

- Remote advertises a manifest version behind `active_epoch`.
- Strict-mode read reports `endpoint_status = stale_remote_epoch_manifest`.
- Matrix action reports `refresh_remote_epoch_manifest`.

## Files

- `src/tests/remote_search/epoch_manifest.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`

`src/tests/remote_search/epoch_manifest.rs` is 1758 lines after this
change, below the 2500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/remote_search/epoch_manifest.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_epoch_manifest_persist_ready --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready` failed before test execution with:
  `undefined symbol: pg_re_throw`.

## Review Needs

Please verify that the updated stale manifest assertion satisfies the
broken-down 12c.2.d task rows and that the tracker evidence wording is
acceptable. The remaining unchecked tracker rows still include the 12c.4
READ schema-drift scope-decision block called out in reviewer feedback.
