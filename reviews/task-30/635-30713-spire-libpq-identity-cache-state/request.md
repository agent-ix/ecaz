# Review Request: SPIRE Libpq Identity Cache State

## Summary

This lands the first Stage C libpq endpoint-identity cache implementation. The
cache is executor-local and bounded by one executor state, so it avoids a
global invalidation surface while removing duplicate endpoint identity preflight
queries across compact candidate receive and remote-heap receive in the same
diagnostic executor pass.

## Changes

- Carries `descriptor_generation` through the internal libpq connection and
  dispatch plan rows so descriptor refreshes are part of the cache key.
- Adds an executor-local endpoint identity cache keyed by coordinator index,
  node, remote index regclass/OID, descriptor generation, descriptor
  `remote_index_identity`, and requested served epoch.
- Stores validated endpoint protocol/version/opclass/storage/profile fields in
  the cache entry, but never stores raw conninfo.
- Adds `ec_spire_remote_search_libpq_identity_cache_summary(...)`, reporting
  dispatch/candidate counts plus endpoint identity query, hit, miss, entry, and
  raw-conninfo-cache counters.
- Routes `ec_spire_remote_pipeline_steps(...)` compact and heap diagnostic
  executor stages through the shared cache summary.
- Extends the PG18 loopback test to prove one miss, one live identity query,
  one cache hit, one cache entry, and `raw_conninfo_cached = false`.
- Updates Phase 11 Stage C with the landed cache-state slice.

## Validation

- `cargo check --no-default-features --features pg18`
  - exit code 0.
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - 1 passed, 0 failed.
- `cargo pgrx test pg18 test_ec_spire_libpq_rejects_identity_mismatch`
  - 1 passed, 0 failed.
- `git diff --check`
  - exit code 0.

Raw logs and command metadata are under `artifacts/`.

## Reviewer Focus

- Confirm the cache key is tight enough for Stage C and does not omit a
  descriptor or epoch dimension needed for correctness.
- Check that the executor-local scope is a reasonable first production step
  before any shared/global cache.
- Check that the pipeline diagnostic reuse is legitimate and does not weaken
  strict endpoint identity rejection.
