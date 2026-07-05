# Review Request: SPIRE Remote Capability Search Gates

## Summary

This slice makes stale remote epoch windows and extension-version skew
pre-dispatch libpq blockers instead of descriptor-only diagnostics.

Before this change, `ec_spire_remote_search_target_readiness` and downstream
libpq surfaces used descriptor state but did not import
`ec_spire_remote_node_capability_plan` status. A remote descriptor could be
active while stale or version-incompatible, and the libpq path would still plan
pipeline dispatch. The executor receive path would only discover later failures
if a remote call was attempted.

## Changes

- Threaded remote capability status into search target readiness for remote
  targets.
- Added exact pre-dispatch blockers:
  - `stale_epoch` / `retention_gap` -> `remote_epoch_window`
  - `incompatible_extension_version` -> `remote_extension_version`
- Blocked libpq pipeline mode before conninfo secret lookup when a remote node
  has a capability failure.
- Propagated the exact blocker through dispatch summary, bind summary, secret
  plan, executor work/readiness, receive-attempt diagnostics, coordinator gate,
  and heap-resolution summary.
- Extended the SQL-visible libpq executor step contract with explicit
  epoch-window and extension-version verification steps.
- Updated the Phase 11 task file with the landed Stage B coverage.

## Validation

Raw logs are in `artifacts/`; metadata is in `artifacts/manifest.md`.

- `cargo pgrx test pg18 test_ec_spire_libpq`
  - 4 passed, including the new strict/degraded capability blocker test.
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
  - 1 passed.
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - 1 passed.
- `git diff --check`
  - exit code 0.

## Reviewer Focus

- Confirm the capability-plan status is the right source of truth for search
  readiness and libpq pre-dispatch blocking.
- Check that blocking before secret lookup is the right behavior for stale
  epoch and extension-version skew.
- Check whether `retention_gap` should remain grouped with
  `remote_epoch_window`.
- Look for any status-precedence regressions in mixed local/remote plans.
