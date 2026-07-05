# Review Request: Production Prepared Statements

Task: 142 — SPIRE Epoch-Keyed Caching
Branch: `task-142-spire-epoch-cache-overhead`
Code commit: `29eca4133da99622a65fb72899052af767f4394f`

## Summary

This slice adds lazy prepared-statement reuse to the async production pooled
connection path.

`SpireRemotePooledConnection` now keeps prepared statement handles for:

- candidate receive
- candidate receive with initial threshold
- normal heap receive
- explicit heap receive after global pre-heap merge
- typed tuple payload receive

The first use on a pooled connection prepares the SQL; later uses on the same
pooled connection call `query` with the cached `tokio_postgres::Statement`
instead of reparsing the SQL text.

## Scope Notes

- This advances Task 142 Phase 2 transport hygiene.
- It only changes the reusable production pooled path; legacy one-shot
  candidate/heap helper paths still use direct SQL text because those
  connections are not retained.
- Remote/session snapshot caching and advisory-lock round-trip reduction remain
  future slices.

## Validation

Packet-local logs:

- `artifacts/cargo-test-production-read-profile-rollup-r3.log`
  - `production_read_profile_row_preserves_metric_rollup ... ok`
- `artifacts/cargo-test-endpoint-identity-cache-regression.log`
  - `cached_production_endpoint_identity_requires_matching_identity ... ok`

`git diff --check` passed locally.

Note: earlier `cargo-test-production-read-profile-rollup*.log` attempts in this
packet were compile-failure iterations while fixing mutability and are
intentionally not committed or cited.
