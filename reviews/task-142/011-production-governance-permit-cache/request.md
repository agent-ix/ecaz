# Task 142 Packet 011: Production Governance Permit Cache

## Summary

This slice moves the production pooled-read governance permit from the per-query
candidate session into `SpireRemotePooledConnection`.

New production pooled connections still acquire
`remote_search_libpq_executor_governance_permit_for_node()` before opening the
remote socket. Reused pooled connections keep their existing permit, so the hot
pooled path avoids the local advisory-lock SPI lock/unlock cycle per node per
query. The permit is released when the pooled connection is dropped, including
pool eviction, closed connection discard, or failed query paths that do not
return the connection to the pool.

Legacy one-shot reads and write paths are unchanged.

## Validation

- `cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture`
  - log: `artifacts/cargo-test-production-read-profile-rollup.log`
  - result: 1 passed, 0 failed; command exit code 0

## Review Notes

Please verify the concurrency semantics: the pooled connection now reserves the
same governance capacity for the lifetime of the retained remote connection
instead of releasing it immediately after each candidate/heap query completes.
This intentionally trades idle pool slot capacity for removing repeated
advisory-lock SPI round trips on reused production remote connections.
