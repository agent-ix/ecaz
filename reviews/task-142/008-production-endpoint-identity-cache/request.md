# Review Request: Production Endpoint Identity Cache

Task: 142 — SPIRE Epoch-Keyed Caching
Branch: `task-142-spire-epoch-cache-overhead`
Code commit: `9626b54d7e3b2738c4f32ba6a92ae7c154faca91`

## Summary

This slice wires endpoint identity reuse into the async production pooled
connection path.

`SpireRemotePooledConnection` now carries:

- the validated remote index OID
- the validated endpoint identity row

On a reused pooled connection, the production candidate request skips both the
per-query `to_regclass` probe and `ec_spire_remote_search_endpoint_identity`
query when the cached endpoint fingerprint still matches the descriptor's
`remote_index_identity`. New connections and mismatched cached identity still
run the existing validation path and populate the connection cache only after a
successful check.

## Scope Notes

- This advances Task 142 Phase 2 transport hygiene.
- It does not change the pool key; the existing key already includes descriptor
  generation, remote index regclass, remote index identity, conninfo fingerprint,
  TLS mode, user, database, and statement timeout.
- Named prepared statements and remote-side snapshot caching remain future
  slices.

## Validation

Packet-local logs:

- `artifacts/cargo-test-production-endpoint-identity-cache-r2.log`
  - `cached_production_endpoint_identity_requires_matching_identity ... ok`
- `artifacts/cargo-test-production-read-profile-rollup.log`
  - `production_read_profile_row_preserves_metric_rollup ... ok`

`git diff --check` passed locally.

Note: `artifacts/cargo-test-production-endpoint-identity-cache.log` is a local
failed compile attempt from before the mutability fix and is intentionally not
committed or cited.
