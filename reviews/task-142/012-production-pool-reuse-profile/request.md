# Task 142 Packet 012: Production Pool Reuse Profile

## Summary

This slice makes production remote connection-pool reuse directly observable in
`ec_spire_remote_search_production_read_profile` and in
`ecaz bench spire-pipeline --include-production-read-profile`.

New metrics:

- `connection_pool_hit_count`
- `connection_pool_miss_count`

The dispatch path increments a hit when `state.take_connection()` hands a
reusable pooled connection into the candidate/heap session, and increments a
miss when the request opens a new remote connection. This lets Task 142 release
A/B packets prove whether production runs are actually reusing pooled
connections instead of inferring reuse from `socket_open_count`.

## Validation

- `cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture`
  - log: `artifacts/cargo-test-production-read-profile-rollup.log`
  - result: 1 passed, 0 failed; command exit code 0
- `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture`
  - log: `artifacts/cargo-test-cli-production-profile-render.log`
  - result: 1 passed, 0 failed; command exit code 0

## Review Notes

The miss counter includes the disabled-pool case because the dispatch path sees
that the same way it sees an empty or non-matching pool: no reusable connection
was provided. This keeps the profile interpretation simple for suite evidence:
hot pooled production reads should show hits and reduced socket opens.
