# Task 39 Review Request: Spire Storage Codec Coverage

Code checkpoint: `a0afb04fe25d7087fb9cc7215704792f5e58133c`

## Summary

This packet raises Task 39 coverage gates for the pgrx-free Spire storage codec
surface.

Changes:

- Added a careful-harness Spire module that reuses the production metadata and
  storage codec source files through `include!` / `#[path]`.
- Reused the existing Spire metadata and storage codec tests in the careful
  lane, plus added local-store-set routing/readback tests.
- Kept relation-backed PG code out of this harness. `relation_plan.rs` and
  `relation_store.rs` remain separate PG-facing gaps.
- Raised 11 Spire storage baseline rows from `0.00%` to their measured careful
  coverage values.

## Evidence

- Focused careful Spire tests:
  `artifacts/careful-spire-storage-tests.log`
  - 101 passed, 0 failed.
- Coverage: `artifacts/coverage/summary.txt`
  - `am/ec_spire/storage/assignment.rs`: `87.14%` line coverage.
  - `am/ec_spire/storage/header.rs`: `82.00%` line coverage.
  - `am/ec_spire/storage/helpers.rs`: `83.50%` line coverage.
  - `am/ec_spire/storage/leaf_v1.rs`: `97.70%` line coverage.
  - `am/ec_spire/storage/leaf_v2.rs`: `71.76%` line coverage.
  - `am/ec_spire/storage/leaf_v2_parts.rs`: `77.52%` line coverage.
  - `am/ec_spire/storage/local_store.rs`: `78.21%` line coverage.
  - `am/ec_spire/storage/local_store_set.rs`: `41.52%` line coverage.
  - `am/ec_spire/storage/routing_delta.rs`: `88.46%` line coverage.
  - `am/ec_spire/storage/top_graph.rs`: `90.32%` line coverage.
  - `am/ec_spire/storage/vec_id.rs`: `69.05%` line coverage.
- Coverage baseline completeness:
  `artifacts/coverage-baseline-check.log`
  - `coverage baseline complete for 40 critical paths`.
- Production compile check: `artifacts/cargo-check-pg18-bench.log`
  - passed with pre-existing warnings.
- Whitespace check: `artifacts/git-diff-check.log`
  - no whitespace errors.

## Review Notes

Please focus on whether the careful harness boundary is appropriate: it covers
pure Spire storage codecs and in-memory local stores, but deliberately does not
claim relation-backed PostgreSQL storage behavior.
