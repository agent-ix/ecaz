# Review Request: SPIRE Libpq Identity Cache Test Matrix

## Summary

This packet processes reviewer feedback from 30712: cache reuse should not
advance without binding PG18 coverage for the key invalidation matrix. The code
commit adds a focused cache-contract probe and expands the capability blocker
fixture so the already-landed executor-local cache is now covered by the
required key and pre-dispatch cases.

## Changes

- Added a pg-test-only identity-cache contract probe that exercises one executor
  state across:
  - initial miss plus insert;
  - exact-key hit;
  - descriptor-generation change miss;
  - served-epoch change miss;
  - descriptor `remote_index_identity` change miss with
    `endpoint_identity_mismatch`.
- Extended the loopback libpq executor test to assert the probe counts:
  3 entries, 4 live identity queries, 1 hit, 4 misses, and exact mismatch
  status.
- Extended capability-block coverage to include `retention_gap` in strict and
  degraded modes.
- Added assertions that stale epoch, retention gap, and extension-version skew
  leave identity-cache entries, queries, hits, misses, compact candidates, and
  heap candidates at zero before dispatch.
- Cross-linked ADR-058 to `plan/design/spire-libpq-identity-cache.md`.
- Updated Phase 11 Stage C with the expanded cache-matrix coverage.

## Validation

- `cargo check --no-default-features --features pg18`
  - exit code 0.
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - 1 passed, 0 failed.
- `cargo pgrx test pg18 test_ec_spire_libpq_capability_blocks`
  - 1 passed, 0 failed.
- `git diff --check`
  - exit code 0.

Raw logs and command metadata are under `artifacts/`.

## Reviewer Focus

- Confirm this closes 30712 P2 enough to continue Stage C cache work.
- Check that the pg-test-only probe is acceptable for key-dimension coverage
  without adding another production SQL surface.
- Check whether any remaining matrix case should still block the next Stage C
  slice, especially degraded live-fingerprint mismatch.
