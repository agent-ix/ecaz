# Task 142 Packet 013: Production Manifest Cache Profile

## Summary

This slice makes production active-epoch manifest cache reuse observable in the
same profile surface used for Task 142 release A/B evidence.

New `ec_spire_remote_search_production_read_profile` metrics:

- `manifest_cache_hit_count`
- `manifest_cache_miss_count`

The existing coordinator fanout manifest cache still owns the behavior. This
change adds a status-returning helper and records whether the production read
path loaded active epoch manifests from that cache or had to populate it. The
`ecaz bench spire-pipeline --include-production-read-profile` table now renders
`manifest_cache_hit_sum` and `manifest_cache_miss_sum`, so later suite packets
can prove manifest snapshot reuse directly.

## Validation

- `cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture`
  - log: `artifacts/cargo-test-production-read-profile-rollup.log`
  - result: 1 passed, 0 failed; command exit code 0
- `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture`
  - log: `artifacts/cargo-test-cli-production-profile-render.log`
  - result: 1 passed, 0 failed; command exit code 0

## Review Notes

This is observability for an existing cache, not a new relation-handle cache.
Object-store relation views still stay request-local because they carry live
PostgreSQL relation/lock lifetime.
