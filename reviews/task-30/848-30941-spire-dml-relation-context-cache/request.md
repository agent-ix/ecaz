---
topic: spire-dml-relation-context-cache
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30941
stage: phase-12.3
status: open
---

# Review Request: SPIRE DML Relation Context Cache

## Scope

Please review commit `0886e3bd6ab873d3f10e8c4bcfc7f8c5e8b33275`
(`Cache SPIRE DML relation context`).

This slice closes the Phase 12.3 relation-context cache and relcache
invalidation items:

- Adds a backend-local cache around the catalog/relcache DML frontdoor relation
  context loader used by planner-hook observation and CustomScan handoff.
- Registers a relcache invalidation callback and evicts cached contexts when
  the heap relid or any index relid observed while building that context is
  invalidated.
- Adds `ec_spire_dml_frontdoor_relation_context_cache()` so hit/miss,
  invalidation, entry-count, and callback-registration state are visible.
- Keeps the SPI-backed relation-context diagnostic path separate from the
  planner-safe catalog/relcache loader.
- Updates the Phase 12 tracker with focused test evidence.

## Review Focus

- Confirm the cache cannot reuse stale relation context after heap/index
  relcache invalidation.
- Confirm watching all index relids from `RelationGetIndexList` is appropriate
  for both the `ec_spire` index and primary-key index dependencies.
- Confirm the diagnostic function is narrow enough for Phase 12.3 and does not
  imply cross-backend or shared-cache behavior.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_dml_context_cache_invalidation_sql`

Key result: `1 passed; 0 failed; 1688 filtered out`.
