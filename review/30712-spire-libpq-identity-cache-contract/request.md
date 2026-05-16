# Review Request: SPIRE Libpq Identity Cache Contract

## Summary

This packet processes reviewer feedback from 30709 and 30710 before Stage C
cache implementation starts. It documents the identity-cache contract that must
hold once validated endpoint identity is cached instead of queried on every
dispatch.

## Changes

- Added `plan/design/spire-libpq-identity-cache.md`.
- Expanded Phase 11 Stage C with explicit cache-key, invalidation, and mismatch
  behavior requirements.
- Captured reviewer requirements:
  - key includes coordinator index, node, remote index, descriptor identity, and
    served epoch;
  - descriptor generation and endpoint protocol/version/opclass/storage/profile
    fingerprint are bound into the entry;
  - descriptor writes, epoch-window changes, fingerprint changes, extension
    changes, storage/profile changes, and local extension upgrade invalidate;
  - live fingerprint mismatch invalidates and reports
    `endpoint_identity_mismatch`; it must not silently reseat descriptor
    identity from the remote endpoint;
  - raw conninfo is not stored in the identity cache.

## Validation

- `git diff --check`
  - exit code 0.

Raw log and manifest are under `artifacts/`.

## Reviewer Focus

- Confirm the cache key is sufficient for Stage C without being overbroad.
- Check whether the invalidation trigger set is complete.
- Check whether backend-local bounded cache is the right first implementation
  target before any shared/global cache.
